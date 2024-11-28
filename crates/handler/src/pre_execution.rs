//! Handles related to the main function of the EVM.
//!
//! They handle initial setup of the EVM, call loop and the final return of the EVM

use bytecode::Bytecode;
use context_interface::{
    journaled_state::JournaledState,
    result::InvalidTransaction,
    transaction::{
        eip7702::Authorization, AccessListTrait, Eip4844Tx, Eip7702Tx, Transaction, TransactionType,
    },
    Block, BlockGetter, Cfg, CfgGetter, JournalStateGetter, JournalStateGetterDBError,
    TransactionGetter,
};
use handler_interface::PreExecutionHandler;
use primitives::{Address, BLOCKHASH_STORAGE_ADDRESS, U256};
use specification::{eip7702, hardfork::SpecId};
use std::{boxed::Box, vec::Vec};

#[derive(Default)]
pub struct EthPreExecution<CTX, ERROR> {
    pub _phantom: core::marker::PhantomData<(CTX, ERROR)>,
}

impl<CTX, ERROR> EthPreExecution<CTX, ERROR> {
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn new_boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

impl<CTX, ERROR> PreExecutionHandler for EthPreExecution<CTX, ERROR>
where
    CTX: TransactionGetter + BlockGetter + JournalStateGetter + CfgGetter,
    ERROR: From<InvalidTransaction> + From<JournalStateGetterDBError<CTX>>,
{
    type Context = CTX;
    type Error = ERROR;

    fn load_accounts(&self, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        let spec = ctx.cfg().spec().into();
        // set journaling state flag.
        ctx.journal().set_spec_id(spec);

        // load coinbase
        // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
        if spec.is_enabled_in(SpecId::SHANGHAI) {
            let coinbase = *ctx.block().beneficiary();
            ctx.journal().warm_account(coinbase);
        }

        // Load blockhash storage address
        // EIP-2935: Serve historical block hashes from state
        if spec.is_enabled_in(SpecId::PRAGUE) {
            ctx.journal().warm_account(BLOCKHASH_STORAGE_ADDRESS);
        }

        // Load access list
        if let Some(access_list) = ctx.tx().access_list().cloned() {
            for access_list in access_list.iter() {
                ctx.journal().warm_account_and_storage(
                    access_list.0,
                    access_list.1.map(|i| U256::from_be_bytes(i.0)),
                )?;
            }
        };

        Ok(())
    }

    fn apply_eip7702_auth_list(&self, ctx: &mut Self::Context) -> Result<u64, Self::Error> {
        let spec = ctx.cfg().spec().into();
        if spec.is_enabled_in(SpecId::PRAGUE) {
            apply_eip7702_auth_list::<CTX, ERROR>(ctx)
        } else {
            Ok(0)
        }
    }

    fn deduct_caller(&self, ctx: &mut Self::Context) -> Result<(), Self::Error> {
        let basefee = *ctx.block().basefee();
        let blob_price = U256::from(ctx.block().blob_gasprice().unwrap_or_default());
        let effective_gas_price = ctx.tx().effective_gas_price(basefee);
        // Subtract gas costs from the caller's account.
        // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.
        let mut gas_cost =
            U256::from(ctx.tx().common_fields().gas_limit()).saturating_mul(effective_gas_price);

        // EIP-4844
        if ctx.tx().tx_type().into() == TransactionType::Eip4844 {
            let blob_gas = U256::from(ctx.tx().eip4844().total_blob_gas());
            gas_cost = gas_cost.saturating_add(blob_price.saturating_mul(blob_gas));
        }

        let is_call = ctx.tx().kind().is_call();
        let caller = ctx.tx().common_fields().caller();

        // load caller's account.
        let caller_account = ctx.journal().load_account(caller)?.data;
        // set new caller account balance.
        caller_account.info.balance = caller_account.info.balance.saturating_sub(gas_cost);

        // bump the nonce for calls. Nonce for CREATE will be bumped in `handle_create`.
        if is_call {
            // Nonce is already checked
            caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);
        }

        // touch account so we know it is changed.
        caller_account.mark_touch();
        Ok(())
    }
}

// /// Main precompile load
// /// TODO Include this inside Wire.
// #[inline]
// pub fn load_precompiles<EvmWiringT: EvmWiring, SPEC: Spec>() -> ContextPrecompiles<EvmWiringT> {
//     ContextPrecompiles::new(PrecompileSpecId::from_spec_id(SPEC::SPEC_ID))
// }

/// Apply EIP-7702 auth list and return number gas refund on already created accounts.
#[inline]
pub fn apply_eip7702_auth_list<
    CTX: TransactionGetter + JournalStateGetter + CfgGetter,
    ERROR: From<InvalidTransaction> + From<JournalStateGetterDBError<CTX>>,
>(
    ctx: &mut CTX,
) -> Result<u64, ERROR> {
    // return if there is no auth list.
    let tx = ctx.tx();
    if tx.tx_type().into() != TransactionType::Eip7702 {
        return Ok(0);
    }

    struct Authorization {
        authority: Option<Address>,
        address: Address,
        nonce: u64,
        chain_id: u64,
    }

    let authorization_list = tx
        .eip7702()
        .authorization_list_iter()
        .map(|a| Authorization {
            authority: a.authority(),
            address: a.address(),
            nonce: a.nonce(),
            chain_id: a.chain_id(),
        })
        .collect::<Vec<_>>();
    let chain_id = ctx.cfg().chain_id();

    let mut refunded_accounts = 0;
    for authorization in authorization_list {
        // 1. recover authority and authorized addresses.
        // authority = ecrecover(keccak(MAGIC || rlp([chain_id, address, nonce])), y_parity, r, s]
        let Some(authority) = authorization.authority else {
            continue;
        };

        // 2. Verify the chain id is either 0 or the chain's current ID.
        if authorization.chain_id != 0 && authorization.chain_id != chain_id {
            continue;
        }

        // warm authority account and check nonce.
        // 3. Add authority to accessed_addresses (as defined in EIP-2929.)
        let mut authority_acc = ctx.journal().load_account_code(authority)?;

        // 4. Verify the code of authority is either empty or already delegated.
        if let Some(bytecode) = &authority_acc.info.code {
            // if it is not empty and it is not eip7702
            if !bytecode.is_empty() && !bytecode.is_eip7702() {
                continue;
            }
        }

        // 5. Verify the nonce of authority is equal to nonce.
        if authorization.nonce != authority_acc.info.nonce {
            continue;
        }

        // 6. Refund the sender PER_EMPTY_ACCOUNT_COST - PER_AUTH_BASE_COST gas if authority exists in the trie.
        if !authority_acc.is_empty() {
            refunded_accounts += 1;
        }

        // 7. Set the code of authority to be 0xef0100 || address. This is a delegation designation.
        let bytecode = Bytecode::new_eip7702(authorization.address);
        authority_acc.info.code_hash = bytecode.hash_slow();
        authority_acc.info.code = Some(bytecode);

        // 8. Increase the nonce of authority by one.
        authority_acc.info.nonce = authority_acc.info.nonce.saturating_add(1);
        authority_acc.mark_touch();
    }

    let refunded_gas =
        refunded_accounts * (eip7702::PER_EMPTY_ACCOUNT_COST - eip7702::PER_AUTH_BASE_COST);

    Ok(refunded_gas)
}

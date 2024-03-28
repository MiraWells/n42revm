//! Handler related to Taiko chain

use crate::{
    handler::{
        mainnet::{self},
        register::EvmHandler,
    },
    interpreter::Gas,
    primitives::{db::Database, spec_to_generic, EVMError, Spec, SpecId, TransactTo, U256},
    Context,
};
extern crate alloc;
use alloc::sync::Arc;
use SpecId::{CANCUN};

pub fn taiko_handle_register<DB: Database, EXT>(handler: &mut EvmHandler<'_, EXT, DB>) {
    spec_to_generic!(handler.cfg.spec_id, {
        handler.pre_execution.deduct_caller = Arc::new(deduct_caller::<SPEC, EXT, DB>);
        handler.post_execution.reimburse_caller = Arc::new(reimburse_caller::<SPEC, EXT, DB>);
        handler.post_execution.reward_beneficiary = Arc::new(reward_beneficiary::<SPEC, EXT, DB>);
    });
}

#[inline]
pub fn reimburse_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    if context.evm.env.tx.taiko.is_anchor {
        return Ok(());
    }
    mainnet::reimburse_caller::<SPEC, EXT, DB>(context, gas)
}

/// Reward beneficiary with gas fee.
#[inline]
pub fn reward_beneficiary<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    if context.evm.env.tx.taiko.is_anchor {
        return Ok(());
    }

    mainnet::reward_beneficiary::<SPEC, EXT, DB>(context, gas)?;

    let treasury = context.evm.env.tx.taiko.treasury;
    let basefee = context.evm.env.block.basefee;

    let (treasury_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(treasury, &mut context.evm.inner.db)?;
    treasury_account.mark_touch();
    treasury_account.info.balance = treasury_account
        .info
        .balance
        .saturating_add(basefee * U256::from(gas.spent() - gas.refunded() as u64));
    Ok(())
}

/// Deduct max balance from caller
#[inline]
pub fn deduct_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // load caller's account.
    let (caller_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(context.evm.inner.env.tx.caller, &mut context.evm.inner.db)?;

    let env = &context.evm.inner.env;

    // Subtract gas costs from the caller's account.
    // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.
    let mut gas_cost = U256::from(env.tx.gas_limit).saturating_mul(env.effective_gas_price());

    // EIP-4844
    if SPEC::enabled(CANCUN) {
        let data_fee = env.calc_data_fee().expect("already checked");
        gas_cost = gas_cost.saturating_add(data_fee);
    }

    if !context.evm.inner.env.tx.taiko.is_anchor {
        // set new caller account balance.
        caller_account.info.balance = caller_account.info.balance.saturating_sub(gas_cost);
    }

    // bump the nonce for calls. Nonce for CREATE will be bumped in `handle_create`.
    if matches!(env.tx.transact_to, TransactTo::Call(_)) {
        // Nonce is already checked
        caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);
    }

    // touch account so we know it is changed.
    caller_account.mark_touch();

    Ok(())
}

use crate::{
    db::{Database, DatabaseRef, EmptyDB, WrapDatabaseRef},
    handler::{register, CfgEnvWithChainSpec, EnvWithChainSpec},
    primitives::{self, CfgEnv, Env, EthChainSpec, InvalidTransaction, TransactionValidation},
    ChainSpec, Context, ContextWithChainSpec, Evm, EvmContext, Handler,
};
use core::marker::PhantomData;
use std::boxed::Box;

/// Evm Builder allows building or modifying EVM.
/// Note that some of the methods that changes underlying structures
/// will reset the registered handler to default mainnet.
pub struct EvmBuilder<'a, BuilderStage, ChainSpecT: ChainSpec, EXT, DB: Database> {
    context: Context<ChainSpecT, EXT, DB>,
    /// Handler that will be used by EVM. It contains handle registers
    handler: Handler<'a, ChainSpecT, Context<ChainSpecT, EXT, DB>, EXT, DB>,
    /// Phantom data to mark the stage of the builder.
    phantom: PhantomData<BuilderStage>,
}

/// First stage of the builder allows setting generic variables.
/// Generic variables are database and external context.
pub struct SetGenericStage;

/// Second stage of the builder allows appending handler registers.
/// Requires the database and external context to be set.
pub struct HandlerStage;

impl<'a> Default for EvmBuilder<'a, SetGenericStage, EthChainSpec, (), EmptyDB> {
    fn default() -> Self {
        Self {
            context: Context::default(),
            handler: EthChainSpec::handler::<'a, (), EmptyDB>(
                <EthChainSpec as primitives::ChainSpec>::Hardfork::default(),
            ),
            phantom: PhantomData,
        }
    }
}

impl<'a, ChainSpecT, EXT, DB: Database> EvmBuilder<'a, SetGenericStage, ChainSpecT, EXT, DB>
where
    ChainSpecT: ChainSpec,
{
    /// Sets the [`ChainSpec`] that will be used by [`Evm`].
    pub fn with_chain_spec<NewChainSpecT>(
        self,
    ) -> EvmBuilder<'a, SetGenericStage, NewChainSpecT, EXT, DB>
    where
        NewChainSpecT: ChainSpec<
            Block: Default,
            Transaction: Default + TransactionValidation<ValidationError: From<InvalidTransaction>>,
        >,
    {
        let Context { evm, external } = self.context;

        EvmBuilder {
            context: Context::new(EvmContext::new(evm.inner.db), external),
            handler: NewChainSpecT::handler::<'a, EXT, DB>(NewChainSpecT::Hardfork::default()),
            phantom: PhantomData,
        }
    }
}

impl<'a, ChainSpecT, EXT, DB: Database> EvmBuilder<'a, SetGenericStage, ChainSpecT, EXT, DB>
where
    ChainSpecT:
        ChainSpec<Transaction: TransactionValidation<ValidationError: From<InvalidTransaction>>>,
{
    /// Sets the [`EmptyDB`] as the [`Database`] that will be used by [`Evm`].
    pub fn with_empty_db(self) -> EvmBuilder<'a, SetGenericStage, ChainSpecT, EXT, EmptyDB> {
        EvmBuilder {
            context: Context::new(
                self.context.evm.with_db(EmptyDB::default()),
                self.context.external,
            ),
            handler: ChainSpecT::handler::<'a, EXT, EmptyDB>(self.handler.spec_id()),
            phantom: PhantomData,
        }
    }
    /// Sets the [`Database`] that will be used by [`Evm`].
    pub fn with_db<ODB: Database>(
        self,
        db: ODB,
    ) -> EvmBuilder<'a, SetGenericStage, ChainSpecT, EXT, ODB> {
        EvmBuilder {
            context: Context::new(self.context.evm.with_db(db), self.context.external),
            handler: ChainSpecT::handler::<'a, EXT, ODB>(self.handler.spec_id()),
            phantom: PhantomData,
        }
    }
    /// Sets the [`DatabaseRef`] that will be used by [`Evm`].
    pub fn with_ref_db<ODB: DatabaseRef>(
        self,
        db: ODB,
    ) -> EvmBuilder<'a, SetGenericStage, ChainSpecT, EXT, WrapDatabaseRef<ODB>> {
        EvmBuilder {
            context: Context::new(
                self.context.evm.with_db(WrapDatabaseRef(db)),
                self.context.external,
            ),
            handler: ChainSpecT::handler::<'a, EXT, WrapDatabaseRef<ODB>>(self.handler.spec_id()),
            phantom: PhantomData,
        }
    }

    /// Sets the external context that will be used by [`Evm`].
    pub fn with_external_context<OEXT>(
        self,
        external: OEXT,
    ) -> EvmBuilder<'a, SetGenericStage, ChainSpecT, OEXT, DB> {
        EvmBuilder {
            context: Context::new(self.context.evm, external),
            handler: ChainSpecT::handler::<'a, OEXT, DB>(self.handler.spec_id()),
            phantom: PhantomData,
        }
    }

    /// Sets Builder with [`EnvWithChainSpec`].
    pub fn with_env_with_handler_cfg(
        mut self,
        env_with_handler_cfg: EnvWithChainSpec<ChainSpecT>,
    ) -> EvmBuilder<'a, HandlerStage, ChainSpecT, EXT, DB> {
        let EnvWithChainSpec { env, spec_id } = env_with_handler_cfg;
        self.context.evm.env = env;
        EvmBuilder {
            context: self.context,
            handler: ChainSpecT::handler::<'a, EXT, DB>(spec_id),
            phantom: PhantomData,
        }
    }

    /// Sets Builder with [`ContextWithChainSpec`].
    pub fn with_context_with_handler_cfg<OEXT, ODB: Database>(
        self,
        context_with_handler_cfg: ContextWithChainSpec<ChainSpecT, OEXT, ODB>,
    ) -> EvmBuilder<'a, HandlerStage, ChainSpecT, OEXT, ODB> {
        EvmBuilder {
            context: context_with_handler_cfg.context,
            handler: ChainSpecT::handler::<'a, OEXT, ODB>(context_with_handler_cfg.spec_id),
            phantom: PhantomData,
        }
    }

    /// Sets Builder with [`CfgEnvWithChainSpec`].
    pub fn with_cfg_env_with_handler_cfg(
        mut self,
        cfg_env_and_spec_id: CfgEnvWithChainSpec<ChainSpecT>,
    ) -> EvmBuilder<'a, HandlerStage, ChainSpecT, EXT, DB> {
        self.context.evm.env.cfg = cfg_env_and_spec_id.cfg_env;

        EvmBuilder {
            context: self.context,
            handler: ChainSpecT::handler::<'a, EXT, DB>(cfg_env_and_spec_id.spec_id),
            phantom: PhantomData,
        }
    }
}

impl<'a, ChainSpecT: ChainSpec, EXT, DB: Database>
    EvmBuilder<'a, HandlerStage, ChainSpecT, EXT, DB>
{
    /// Creates new builder from Evm, Evm is consumed and all field are moved to Builder.
    /// It will preserve set handler and context.
    ///
    /// Builder is in HandlerStage and both database and external are set.
    pub fn new(evm: Evm<'a, ChainSpecT, EXT, DB>) -> Self {
        Self {
            context: evm.context,
            handler: evm.handler,
            phantom: PhantomData,
        }
    }
}

impl<'a, ChainSpecT: ChainSpec, EXT, DB: Database> EvmBuilder<'a, HandlerStage, ChainSpecT, EXT, DB>
where
    ChainSpecT:
        ChainSpec<Transaction: TransactionValidation<ValidationError: From<InvalidTransaction>>>,
{
    /// Sets the [`EmptyDB`] and resets the [`Handler`] to default mainnet.
    pub fn reset_handler_with_empty_db(
        self,
    ) -> EvmBuilder<'a, HandlerStage, ChainSpecT, EXT, EmptyDB> {
        EvmBuilder {
            context: Context::new(
                self.context.evm.with_db(EmptyDB::default()),
                self.context.external,
            ),
            handler: ChainSpecT::handler::<'a, EXT, EmptyDB>(self.handler.spec_id()),
            phantom: PhantomData,
        }
    }

    /// Sets the [`Database`] that will be used by [`Evm`]
    /// and resets the [`Handler`] to default mainnet.
    pub fn reset_handler_with_db<ODB: Database>(
        self,
        db: ODB,
    ) -> EvmBuilder<'a, SetGenericStage, ChainSpecT, EXT, ODB> {
        EvmBuilder {
            context: Context::new(self.context.evm.with_db(db), self.context.external),
            handler: ChainSpecT::handler::<'a, EXT, ODB>(self.handler.spec_id()),
            phantom: PhantomData,
        }
    }

    /// Resets [`Handler`] and sets the [`DatabaseRef`] that will be used by [`Evm`]
    /// and resets the [`Handler`] to default mainnet.
    pub fn reset_handler_with_ref_db<ODB: DatabaseRef>(
        self,
        db: ODB,
    ) -> EvmBuilder<'a, SetGenericStage, ChainSpecT, EXT, WrapDatabaseRef<ODB>> {
        EvmBuilder {
            context: Context::new(
                self.context.evm.with_db(WrapDatabaseRef(db)),
                self.context.external,
            ),
            handler: ChainSpecT::handler::<'a, EXT, WrapDatabaseRef<ODB>>(self.handler.spec_id()),
            phantom: PhantomData,
        }
    }

    /// Resets [`Handler`] and sets new `ExternalContext` type.
    ///  and resets the [`Handler`] to default mainnet.
    pub fn reset_handler_with_external_context<OEXT>(
        self,
        external: OEXT,
    ) -> EvmBuilder<'a, SetGenericStage, ChainSpecT, OEXT, DB> {
        EvmBuilder {
            context: Context::new(self.context.evm, external),
            handler: ChainSpecT::handler::<'a, OEXT, DB>(self.handler.spec_id()),
            phantom: PhantomData,
        }
    }
}

impl<'a, BuilderStage, ChainSpecT: ChainSpec, EXT, DB: Database>
    EvmBuilder<'a, BuilderStage, ChainSpecT, EXT, DB>
{
    /// This modifies the [EvmBuilder] to make it easy to construct an [`Evm`] with a _specific_
    /// handler.
    ///
    /// # Example
    /// ```rust
    /// use revm::{EvmBuilder, EvmHandler, db::EmptyDB, primitives::{EthChainSpec, SpecId}};
    /// use revm_interpreter::primitives::CancunSpec;
    /// let builder = EvmBuilder::default();
    ///
    /// // get the desired handler
    /// let mainnet = EvmHandler::<'_, EthChainSpec, (), EmptyDB>::mainnet_with_spec(SpecId::CANCUN);
    /// let builder = builder.with_handler(mainnet);
    ///
    /// // build the EVM
    /// let evm = builder.build();
    /// ```
    pub fn with_handler(
        self,
        handler: Handler<'a, ChainSpecT, Context<ChainSpecT, EXT, DB>, EXT, DB>,
    ) -> EvmBuilder<'a, BuilderStage, ChainSpecT, EXT, DB> {
        EvmBuilder {
            context: self.context,
            handler,
            phantom: PhantomData,
        }
    }

    /// Builds the [`Evm`].
    pub fn build(self) -> Evm<'a, ChainSpecT, EXT, DB> {
        Evm::new(self.context, self.handler)
    }

    /// Register Handler that modifies the behavior of EVM.
    /// Check [`Handler`] for more information.
    ///
    /// When called, EvmBuilder will transition from SetGenericStage to HandlerStage.
    pub fn append_handler_register(
        mut self,
        handle_register: register::HandleRegister<ChainSpecT, EXT, DB>,
    ) -> EvmBuilder<'a, HandlerStage, ChainSpecT, EXT, DB> {
        self.handler
            .append_handler_register(register::HandleRegisters::Plain(handle_register));
        EvmBuilder {
            context: self.context,
            handler: self.handler,

            phantom: PhantomData,
        }
    }

    /// Register Handler that modifies the behavior of EVM.
    /// Check [`Handler`] for more information.
    ///
    /// When called, EvmBuilder will transition from SetGenericStage to HandlerStage.
    pub fn append_handler_register_box(
        mut self,
        handle_register: register::HandleRegisterBox<'a, ChainSpecT, EXT, DB>,
    ) -> EvmBuilder<'a, HandlerStage, ChainSpecT, EXT, DB> {
        self.handler
            .append_handler_register(register::HandleRegisters::Box(handle_register));
        EvmBuilder {
            context: self.context,
            handler: self.handler,

            phantom: PhantomData,
        }
    }

    /// Allows modification of Evm Database.
    pub fn modify_db(mut self, f: impl FnOnce(&mut DB)) -> Self {
        f(&mut self.context.evm.db);
        self
    }

    /// Allows modification of external context.
    pub fn modify_external_context(mut self, f: impl FnOnce(&mut EXT)) -> Self {
        f(&mut self.context.external);
        self
    }

    /// Allows modification of Evm Environment.
    pub fn modify_env(mut self, f: impl FnOnce(&mut Box<Env<ChainSpecT>>)) -> Self {
        f(&mut self.context.evm.env);
        self
    }

    /// Sets Evm Environment.
    pub fn with_env(mut self, env: Box<Env<ChainSpecT>>) -> Self {
        self.context.evm.env = env;
        self
    }

    /// Allows modification of Evm's Transaction Environment.
    pub fn modify_tx_env(mut self, f: impl FnOnce(&mut ChainSpecT::Transaction)) -> Self {
        f(&mut self.context.evm.env.tx);
        self
    }

    /// Sets Evm's Transaction Environment.
    pub fn with_tx_env(mut self, tx_env: ChainSpecT::Transaction) -> Self {
        self.context.evm.env.tx = tx_env;
        self
    }

    /// Allows modification of Evm's Block Environment.
    pub fn modify_block_env(mut self, f: impl FnOnce(&mut ChainSpecT::Block)) -> Self {
        f(&mut self.context.evm.env.block);
        self
    }

    /// Sets Evm's Block Environment.
    pub fn with_block_env(mut self, block_env: ChainSpecT::Block) -> Self {
        self.context.evm.env.block = block_env;
        self
    }

    /// Allows modification of Evm's Config Environment.
    pub fn modify_cfg_env(mut self, f: impl FnOnce(&mut CfgEnv)) -> Self {
        f(&mut self.context.evm.env.cfg);
        self
    }
}

impl<'a, BuilderStage, ChainSpecT, EXT, DB> EvmBuilder<'a, BuilderStage, ChainSpecT, EXT, DB>
where
    ChainSpecT: ChainSpec<Block: Default>,
    DB: Database,
{
    /// Clears Block environment of EVM.
    pub fn with_clear_block_env(mut self) -> Self {
        self.context.evm.env.block = ChainSpecT::Block::default();
        self
    }
}

impl<'a, BuilderStage, ChainSpecT, EXT, DB> EvmBuilder<'a, BuilderStage, ChainSpecT, EXT, DB>
where
    ChainSpecT: ChainSpec<Transaction: Default>,
    DB: Database,
{
    /// Clears Transaction environment of EVM.
    pub fn with_clear_tx_env(mut self) -> Self {
        self.context.evm.env.tx = ChainSpecT::Transaction::default();
        self
    }
}

impl<'a, BuilderStage, ChainSpecT, EXT, DB> EvmBuilder<'a, BuilderStage, ChainSpecT, EXT, DB>
where
    ChainSpecT: ChainSpec<Block: Default, Transaction: Default>,
    DB: Database,
{
    /// Clears Environment of EVM.
    pub fn with_clear_env(mut self) -> Self {
        self.context.evm.env.clear();
        self
    }
}

impl<'a, BuilderStage, ChainSpecT: ChainSpec, EXT, DB: Database>
    EvmBuilder<'a, BuilderStage, ChainSpecT, EXT, DB>
where
    ChainSpecT:
        ChainSpec<Transaction: TransactionValidation<ValidationError: From<InvalidTransaction>>>,
{
    /// Sets specification Id , that will mark the version of EVM.
    /// It represent the hard fork of ethereum.
    ///
    /// # Note
    ///
    /// When changed it will reapply all handle registers, this can be
    /// expensive operation depending on registers.
    pub fn with_spec_id(mut self, spec_id: ChainSpecT::Hardfork) -> Self {
        self.handler.modify_spec_id(spec_id);
        EvmBuilder {
            context: self.context,
            handler: self.handler,

            phantom: PhantomData,
        }
    }

    /// Resets [`Handler`] to default mainnet.
    pub fn reset_handler(mut self) -> Self {
        self.handler = ChainSpecT::handler::<'a, EXT, DB>(self.handler.spec_id());
        self
    }
}

#[cfg(test)]
mod test {
    use crate::{
        db::EmptyDB,
        inspector::inspector_handle_register,
        inspectors::NoOpInspector,
        primitives::{
            address, AccountInfo, Address, Bytecode, Bytes, PrecompileResult, SpecId, TxKind, U256,
        },
        Context, ContextPrecompile, ContextStatefulPrecompile, Evm, InMemoryDB, InnerEvmContext,
    };
    use revm_interpreter::{gas, Host, Interpreter};
    use revm_precompile::PrecompileOutput;
    use std::{cell::RefCell, rc::Rc, sync::Arc};

    type TestChainSpec = crate::primitives::EthChainSpec;

    /// Custom evm context
    #[derive(Default, Clone, Debug)]
    pub(crate) struct CustomContext {
        pub(crate) inner: Rc<RefCell<u8>>,
    }

    #[test]
    fn simple_add_stateful_instruction() {
        let code = Bytecode::new_raw([0xED, 0x00].into());
        let code_hash = code.hash_slow();
        let to_addr = address!("ffffffffffffffffffffffffffffffffffffffff");

        // initialize the custom context and make sure it's zero
        let custom_context = CustomContext::default();
        assert_eq!(*custom_context.inner.borrow(), 0);

        let to_capture = custom_context.clone();
        let mut evm = Evm::builder()
            .with_chain_spec::<TestChainSpec>()
            .with_db(InMemoryDB::default())
            .modify_db(|db| {
                db.insert_account_info(to_addr, AccountInfo::new(U256::ZERO, 0, code_hash, code))
            })
            .modify_tx_env(|tx| {
                let transact_to = &mut tx.transact_to;

                *transact_to = TxKind::Call(to_addr)
            })
            // we need to use handle register box to capture the custom context in the handle
            // register
            .append_handler_register_box(Box::new(move |handler| {
                let custom_context = to_capture.clone();

                // we need to use a box to capture the custom context in the instruction
                let custom_instruction = Box::new(
                    move |_interp: &mut Interpreter,
                          _host: &mut Context<TestChainSpec, (), InMemoryDB>| {
                        // modify the value
                        let mut inner = custom_context.inner.borrow_mut();
                        *inner += 1;
                    },
                );

                // need to  ensure the instruction table is a boxed instruction table so that we
                // can insert the custom instruction as a boxed instruction
                handler
                    .instruction_table
                    .insert_boxed(0xED, custom_instruction);
            }))
            .build();

        let _result_and_state = evm.transact().unwrap();

        // ensure the custom context was modified
        assert_eq!(*custom_context.inner.borrow(), 1);
    }

    #[test]
    fn simple_add_instruction() {
        const CUSTOM_INSTRUCTION_COST: u64 = 133;
        const INITIAL_TX_GAS: u64 = 21000;
        const EXPECTED_RESULT_GAS: u64 = INITIAL_TX_GAS + CUSTOM_INSTRUCTION_COST;

        fn custom_instruction(interp: &mut Interpreter, _host: &mut impl Host) {
            // just spend some gas
            gas!(interp, CUSTOM_INSTRUCTION_COST);
        }

        let code = Bytecode::new_raw([0xED, 0x00].into());
        let code_hash = code.hash_slow();
        let to_addr = address!("ffffffffffffffffffffffffffffffffffffffff");

        let mut evm = Evm::builder()
            .with_chain_spec::<TestChainSpec>()
            .with_db(InMemoryDB::default())
            .modify_db(|db| {
                db.insert_account_info(to_addr, AccountInfo::new(U256::ZERO, 0, code_hash, code))
            })
            .modify_tx_env(|tx| {
                let transact_to = &mut tx.transact_to;

                *transact_to = TxKind::Call(to_addr)
            })
            .append_handler_register(|handler| {
                handler.instruction_table.insert(0xED, custom_instruction)
            })
            .build();

        let result_and_state = evm.transact().unwrap();
        assert_eq!(result_and_state.result.gas_used(), EXPECTED_RESULT_GAS);
    }

    #[test]
    fn simple_build() {
        // build without external with latest spec
        Evm::builder().with_chain_spec::<TestChainSpec>().build();
        // build with empty db
        Evm::builder()
            .with_chain_spec::<TestChainSpec>()
            .with_empty_db()
            .build();
        // build with_db
        Evm::builder()
            .with_chain_spec::<TestChainSpec>()
            .with_db(EmptyDB::default())
            .build();
        // build with empty external
        Evm::builder()
            .with_chain_spec::<TestChainSpec>()
            .with_empty_db()
            .build();
        // build with some external
        Evm::builder()
            .with_chain_spec::<TestChainSpec>()
            .with_empty_db()
            .with_external_context(())
            .build();
        // build with spec
        Evm::builder()
            .with_empty_db()
            .with_spec_id(SpecId::HOMESTEAD)
            .build();

        // with with Env change in multiple places
        Evm::builder()
            .with_chain_spec::<TestChainSpec>()
            .with_empty_db()
            .modify_tx_env(|tx| tx.gas_limit = 10)
            .build();
        Evm::builder()
            .with_chain_spec::<TestChainSpec>()
            .modify_tx_env(|tx| tx.gas_limit = 10)
            .build();
        Evm::builder()
            .with_chain_spec::<TestChainSpec>()
            .with_empty_db()
            .modify_tx_env(|tx| tx.gas_limit = 10)
            .build();
        Evm::builder()
            .with_chain_spec::<TestChainSpec>()
            .with_empty_db()
            .modify_tx_env(|tx| tx.gas_limit = 10)
            .build();

        // with inspector handle
        Evm::builder()
            .with_chain_spec::<TestChainSpec>()
            .with_empty_db()
            .with_external_context(NoOpInspector)
            .append_handler_register(inspector_handle_register)
            .build();

        // create the builder
        let evm = Evm::builder()
            .with_db(EmptyDB::default())
            .with_chain_spec::<TestChainSpec>()
            .with_external_context(NoOpInspector)
            .append_handler_register(inspector_handle_register)
            // this would not compile
            // .with_db(..)
            .build();

        let Context { external: _, .. } = evm.into_context();
    }

    #[test]
    fn build_modify_build() {
        // build evm
        let evm = Evm::builder()
            .with_empty_db()
            .with_spec_id(SpecId::HOMESTEAD)
            .build();

        // modify evm
        let evm = evm.modify().with_spec_id(SpecId::FRONTIER).build();
        let _ = evm
            .modify()
            .modify_tx_env(|tx| tx.chain_id = Some(2))
            .build();
    }

    #[test]
    fn build_custom_precompile() {
        struct CustomPrecompile;

        impl ContextStatefulPrecompile<TestChainSpec, EmptyDB> for CustomPrecompile {
            fn call(
                &self,
                _input: &Bytes,
                _gas_limit: u64,
                _context: &mut InnerEvmContext<TestChainSpec, EmptyDB>,
            ) -> PrecompileResult {
                Ok(PrecompileOutput::new(10, Bytes::new()))
            }
        }

        let spec_id = crate::primitives::SpecId::HOMESTEAD;

        let mut evm = Evm::builder()
            .with_chain_spec::<TestChainSpec>()
            .with_spec_id(spec_id)
            .append_handler_register(|handler| {
                let precompiles = handler.pre_execution.load_precompiles();
                handler.pre_execution.load_precompiles = Arc::new(move || {
                    let mut precompiles = precompiles.clone();
                    precompiles.extend([(
                        Address::ZERO,
                        ContextPrecompile::ContextStateful(Arc::new(CustomPrecompile)),
                    )]);
                    precompiles
                });
            })
            .build();

        evm.transact().unwrap();
    }
}

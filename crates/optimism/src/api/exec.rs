use crate::{
    evm::OpEvm, handler::OpHandler, transaction::OpTxTr, L1BlockInfo, OpHaltReason, OpSpecId,
    OpTransactionError,
};
use inspector::{InspectCommitEvm, InspectEvm, Inspector, JournalExt};
use precompile::Log;
use revm::{
    context_interface::{
        result::{EVMError, ExecutionResult, ResultAndState},
        Block, Cfg, ContextTr, Database, Journal,
    },
    handler::{handler::EvmTr, instructions::EthInstructions, EthFrame, Handler},
    interpreter::interpreter::EthInterpreter,
    state::EvmState,
    Context, DatabaseCommit, ExecuteCommitEvm, ExecuteEvm,
};
use std::vec::Vec;

impl<BLOCK, TX, CFG, DB, JOURNAL, INSP> ExecuteEvm
    for OpEvm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>>,
    >
where
    BLOCK: Block,
    TX: OpTxTr,
    CFG: Cfg<Spec = OpSpecId>,
    DB: Database,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
{
    type Output =
        Result<ResultAndState<OpHaltReason>, EVMError<<DB as Database>::Error, OpTransactionError>>;

    fn transact_previous(&mut self) -> Self::Output {
        let mut h = OpHandler::<_, _, EthFrame<_, _, _>>::new();
        h.run(self)
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, INSP> ExecuteCommitEvm
    for OpEvm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>>,
    >
where
    BLOCK: Block,
    TX: OpTxTr,
    CFG: Cfg<Spec = OpSpecId>,
    DB: Database + DatabaseCommit,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
{
    type CommitOutput = Result<
        ExecutionResult<OpHaltReason>,
        EVMError<<DB as Database>::Error, OpTransactionError>,
    >;

    fn transact_commit_previous(&mut self) -> Self::CommitOutput {
        self.transact_previous().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, INSP> InspectEvm
    for OpEvm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>>,
    >
where
    BLOCK: Block,
    TX: OpTxTr,
    CFG: Cfg<Spec = OpSpecId>,
    DB: Database,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
    INSP: Inspector<Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>, EthInterpreter>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.0.data.inspector = inspector;
    }

    fn inspect_previous(&mut self) -> Self::Output {
        let mut h = OpHandler::<_, _, EthFrame<_, _, _>>::new();
        h.run(self)
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, INSP> InspectCommitEvm
    for OpEvm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>>,
    >
where
    BLOCK: Block,
    TX: OpTxTr,
    CFG: Cfg<Spec = OpSpecId>,
    DB: Database + DatabaseCommit,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
    INSP: Inspector<Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>, EthInterpreter>,
{
    fn inspect_commit_previous(&mut self) -> Self::CommitOutput {
        self.inspect_previous().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}

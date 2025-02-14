use revm::{
    context::{setters::ContextSetters, Evm},
    context_interface::{ContextTr, Journal},
    handler::{
        instructions::EthInstructions, EthFrame, EvmTr, EvmTrError, Frame, FrameResult, Handler,
        MainnetHandler, PrecompileProvider,
    },
    interpreter::{interpreter::EthInterpreter, FrameInput, InterpreterResult},
    primitives::Log,
    state::EvmState,
    DatabaseCommit,
};
use std::vec::Vec;

use crate::{
    InspectCommitEvm, InspectEvm, Inspector, InspectorEvmTr, InspectorFrame, InspectorHandler,
    JournalExt,
};

impl<EVM, ERROR, FRAME> InspectorHandler for MainnetHandler<EVM, ERROR, FRAME>
where
    EVM: InspectorEvmTr<
        Context: ContextTr<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>,
        Inspector: Inspector<<<Self as Handler>::Evm as EvmTr>::Context, EthInterpreter>,
    >,
    ERROR: EvmTrError<EVM>,
    FRAME: Frame<Evm = EVM, Error = ERROR, FrameResult = FrameResult, FrameInit = FrameInput>
        + InspectorFrame<IT = EthInterpreter>,
{
    type IT = EthInterpreter;
}

impl<CTX, INSP, PRECOMPILES> InspectEvm
    for Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILES>
where
    CTX: ContextSetters
        + ContextTr<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)> + JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
    PRECOMPILES: PrecompileProvider<Context = CTX, Output = InterpreterResult>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.data.inspector = inspector;
    }

    fn inspect_previous(&mut self) -> Self::Output {
        let mut t = MainnetHandler::<_, _, EthFrame<_, _, _>> {
            _phantom: core::marker::PhantomData,
        };

        t.inspect_run(self)
    }
}

impl<CTX, INSP, PRECOMPILES> InspectCommitEvm
    for Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILES>
where
    CTX: ContextSetters
        + ContextTr<
            Journal: Journal<FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
            Db: DatabaseCommit,
        >,
    INSP: Inspector<CTX, EthInterpreter>,
    PRECOMPILES: PrecompileProvider<Context = CTX, Output = InterpreterResult>,
{
    fn inspect_commit_previous(&mut self) -> Self::CommitOutput {
        self.inspect_previous().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}

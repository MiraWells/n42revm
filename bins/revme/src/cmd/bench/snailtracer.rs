use database::BenchmarkDB;
use revm::{
    bytecode::Bytecode,
    handler::EthHandler,
    primitives::{address, bytes, Bytes, TxKind},
    Context, MainEvm,
};

pub fn simple_example() {
    let bytecode = Bytecode::new_raw(CONTRACT_DATA.clone());

    let context = Context::builder()
        .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
        .modify_tx_chained(|tx| {
            // execution globals block hash/gas_limit/coinbase/timestamp..
            tx.caller = address!("1000000000000000000000000000000000000000");
            tx.transact_to = TxKind::Call(address!("0000000000000000000000000000000000000000"));
            tx.data = bytes!("30627b7c");
        });
    let mut evm = MainEvm::new(context, EthHandler::default());

    let _ = evm.transact().unwrap();
}

pub fn run() {
    println!("Running snailtracer example!");
    let start = std::time::Instant::now();
    simple_example();
    println!("elapsed: {:?}", start.elapsed());
}

const CONTRACT_DATA: Bytes = Bytes::from_static(include_str!("snailtracer.hex").as_bytes());

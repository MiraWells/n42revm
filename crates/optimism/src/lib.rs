//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod api;
pub mod bn128;
pub mod evm;
pub mod fast_lz;
pub mod handler;
pub mod l1block;
pub mod result;
pub mod spec;
pub mod transaction;

pub use api::{
    builder::{OpBuilder, OpContext},
    default_ctx::DefaultOp,
};
pub use evm::OpEvm;
pub use l1block::{L1BlockInfo, BASE_FEE_RECIPIENT, L1_BLOCK_CONTRACT, L1_FEE_RECIPIENT};
pub use result::OpHaltReason;
pub use spec::*;
pub use transaction::{error::OpTransactionError, estimate_tx_compressed_size, OpTransaction};

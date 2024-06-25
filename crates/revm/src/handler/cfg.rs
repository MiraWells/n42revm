use core::ops::{Deref, DerefMut};
use std::boxed::Box;

use crate::primitives::{CfgEnv, ChainSpec, Env};

/// Configuration environment with the chain spec id.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CfgEnvWithChainSpec<ChainSpecT: ChainSpec> {
    /// Configuration environment.
    pub cfg_env: CfgEnv,
    /// Handler configuration fields.
    pub spec_id: ChainSpecT::Hardfork,
}

impl<ChainSpecT: ChainSpec> CfgEnvWithChainSpec<ChainSpecT> {
    /// Returns new instance of `CfgEnvWithHandlerCfg`.
    pub fn new(cfg_env: CfgEnv, spec_id: ChainSpecT::Hardfork) -> Self {
        Self { cfg_env, spec_id }
    }
}

impl<ChainSpecT: ChainSpec> DerefMut for CfgEnvWithChainSpec<ChainSpecT> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cfg_env
    }
}

impl<ChainSpecT: ChainSpec> Deref for CfgEnvWithChainSpec<ChainSpecT> {
    type Target = CfgEnv;

    fn deref(&self) -> &Self::Target {
        &self.cfg_env
    }
}

/// Evm environment with the chain spec id.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnvWithChainSpec<ChainSpecT: ChainSpec> {
    /// Evm enironment.
    pub env: Box<Env<ChainSpecT>>,
    /// Handler configuration fields.
    pub spec_id: ChainSpecT::Hardfork,
}

impl<ChainSpecT: ChainSpec> EnvWithChainSpec<ChainSpecT> {
    /// Returns new `EnvWithHandlerCfg` instance.
    pub fn new(env: Box<Env<ChainSpecT>>, spec_id: ChainSpecT::Hardfork) -> Self {
        Self { env, spec_id }
    }

    /// Takes `CfgEnvWithHandlerCfg` and returns new `EnvWithHandlerCfg` instance.
    pub fn new_with_cfg_env(
        cfg: CfgEnvWithChainSpec<ChainSpecT>,
        block: ChainSpecT::Block,
        tx: ChainSpecT::Transaction,
    ) -> Self {
        Self::new(Env::boxed(cfg.cfg_env, block, tx), cfg.spec_id)
    }

    /// Returns the specification id.
    pub const fn spec_id(&self) -> ChainSpecT::Hardfork {
        self.spec_id
    }
}

impl<ChainSpecT: ChainSpec> DerefMut for EnvWithChainSpec<ChainSpecT> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.env
    }
}

impl<ChainSpecT: ChainSpec> Deref for EnvWithChainSpec<ChainSpecT> {
    type Target = Env<ChainSpecT>;

    fn deref(&self) -> &Self::Target {
        &self.env
    }
}

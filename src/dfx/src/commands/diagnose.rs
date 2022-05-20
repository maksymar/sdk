use clap::Parser;
use tokio::runtime::Runtime;

use crate::lib::{
    environment::Environment, error::DfxResult, migrate::migrate,
    provider::create_agent_environment,
};

/// Detects known problems in the current environment caused by upgrading DFX, and suggests commands to fix them.
/// These commands can be batch-run automatically via `dfx fix`.
#[derive(Parser)]
pub struct DiagnoseOpts {
    #[clap(long)]
    network: Option<String>,
}

pub fn exec(env: &dyn Environment, opts: DiagnoseOpts) -> DfxResult {
    let env = create_agent_environment(env, opts.network)?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime
        .block_on(async { migrate(&env, env.get_network_descriptor().unwrap(), false).await })?;
    Ok(())
}
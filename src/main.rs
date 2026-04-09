mod cli;
mod core;
mod core_extensions;
mod core_integrate;
mod core_tag;
mod error;
mod snapshot;
mod mirror;
mod ssh;
mod duration;
mod integrate;
mod tag;

use anyhow::Result;
use cli::Cli;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.execute()?;

    Ok(())
}

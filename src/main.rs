mod alias;
mod cli;
mod core;
mod core_extensions;
mod core_integrate;
mod core_tag;
mod duration;
mod error;
mod integrate;
mod mirror;
mod snapshot;
mod ssh;
mod tag;
mod toriignore;

use anyhow::Result;
use cli::Cli;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.execute()?;

    Ok(())
}

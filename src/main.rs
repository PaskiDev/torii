mod cli;
mod config;
mod core;
mod core_extensions;
mod core_tag;
mod duration;
mod error;
mod mirror;
mod remote;
mod scanner;
mod snapshot;
mod ssh;
mod tag;
mod toriignore;
mod versioning;

use anyhow::Result;
use cli::Cli;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.execute()?;

    Ok(())
}

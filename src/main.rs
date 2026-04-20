// Copyright (c) 2026 Torii Project. All Rights Reserved.
// Licensed under the Torii Source-Available License (Non-Commercial Fork-Friendly) v1.0.
// See LICENSE file in the project root for full license information.
// Commercial use is prohibited without explicit written permission from the copyright holder.

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
mod tui;
mod workspace;

use anyhow::Result;
use cli::Cli;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.execute()?;

    Ok(())
}

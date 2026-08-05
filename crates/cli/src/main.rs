use std::ffi::OsString;
use std::net::SocketAddr;
use std::time::Duration;

mod commands;
mod python;
mod tui;

use anyhow::Context;
use clap::{Parser, Subcommand};

use tracing::info;
use zygo_core::store::{MemoryStore, Store};

use crate::commands::run_workflow;

#[derive(Parser)]
#[command(name = "zygo", about = "Zygo CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run a workflow given a target and fsspec URI
    Run {
        target: String,
        fsspec_uri: String,
        workers: Option<usize>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run {
            target,
            fsspec_uri,
            workers,
        } => run_workflow(&target, &fsspec_uri, workers).await,
    }
}

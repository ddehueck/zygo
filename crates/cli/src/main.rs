mod commands;
mod python;
mod tui;

use clap::{Parser, Subcommand};

use crate::commands::{list_workflow_runs, nuke_database, run_workflow};

#[derive(Parser)]
#[command(name = "zygo", about = "Zygo CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List workflow runs in creation order
    Ls {
        /// Filter runs by a tag formatted as KEY=VALUE
        #[arg(long, value_name = "KEY=VALUE")]
        filter: Option<String>,
    },

    /// Delete the local database after confirmation
    Nuke,

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
        Command::Ls { filter } => list_workflow_runs(filter.as_deref()).await,
        Command::Nuke => nuke_database(),
        Command::Run {
            target,
            fsspec_uri,
            workers,
        } => run_workflow(&target, &fsspec_uri, workers).await,
    }
}

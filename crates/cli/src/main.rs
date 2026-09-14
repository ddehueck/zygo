mod commands;
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
        path: String,
        #[arg(long)]
        workers: Option<usize>,
        /// Ignore cached job results and execute jobs again.
        #[arg(long)]
        disable_cache: bool,
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
            path,
            workers,
            disable_cache,
        } => run_workflow(&target, &path, workers, disable_cache).await,
    }
}

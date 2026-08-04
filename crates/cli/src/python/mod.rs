//! These commands interact with the zygo python library.
//! They allow the CLI to retrieve metadata and run workflow jobs.

mod types;

use types::JobRunArgs;
pub use types::WorkflowMetadata;

enum PythonCommand {
    GetMetadata(GetMetadataArgs),
    RunJob(RunJobArgs),
}

struct GetMetadataArgs {
    target: String,
}

struct RunJobArgs {
    target: String,
    args: JobRunArgs, // TODO: Decide on JobRun or RunJob!
}

// pub fn materialize_python_command(cmd: PythonCommand) -> Comma {}

use serde::{Deserialize, Serialize};

use crate::ipc;

// Describes a shell-free command prefix used to launch a Zygo IPC CLI.
// The runtime appends an IPC subcommand (`run` or `metadata`) and its arguments.
//
// Examples:
//
// Direct Python:
//   exec = "python"
//   args = ["-m", "zygo._internal.ipc.v0"]
//
// Through uv:
//   exec = "uv"
//   args = ["run", "python", "-m", "zygo._internal.ipc.v0"]
//
// Through Docker:
//   exec = "docker"
//   args = [
//       "run", "--rm", "-i",
//       "--volume", "/host/myapp:/app",
//       "--workdir", "/app",
//       "my-zygo-image",
//       "python", "-m", "zygo._internal.ipc.v0",
//   ]
//
// For example, invoking `run` through Docker produces:
//   docker run --rm -i ... my-zygo-image \
//       python -m zygo._internal.ipc.v0 run ...
//
// Keeping the executable and arguments separate avoids shell parsing and quoting
// issues while supporting other wrappers such as Podman or Nix.
//
// These should impl some common interface to check if they work in the remote orchestrator mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Entrypoint {
    // TODO: These should implement a run cmd?
    Python(ipc::v0::PythonCli),
}

impl Entrypoint {
    pub fn cwd(&self) -> &str {
        match self {
            Self::Python(entrypoint) => entrypoint.cwd(),
        }
    }
}

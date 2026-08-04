use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalEntrypoint {
    pub cwd: String,
    pub exec: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteEntrypoint {
    pub url: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobEntrypoint {
    Local(LocalEntrypoint),
    Remote(RemoteEntrypoint),
}

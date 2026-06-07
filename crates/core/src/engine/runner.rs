use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use crate::models::JobArgs;
use crate::models::job_entrypoint::{JobEntrypoint, LocalEntrypoint, RemoteEntrypoint};
use crate::store::{StorageProvider, Store};

pub struct Runner<S: StorageProvider> {
    _store: Arc<Store<S>>,
}

impl<S: StorageProvider> Runner<S> {
    pub fn new(store: Arc<Store<S>>) -> Self {
        Self { _store: store }
    }

    pub async fn execute(
        &self,
        job_args: JobArgs,
        job_entrypoint: JobEntrypoint,
    ) -> Result<(), anyhow::Error> {
        match job_entrypoint {
            JobEntrypoint::Local(local) => Self::execute_local(job_args, local).await,
            JobEntrypoint::Remote(remote) => Self::execute_remote(job_args, remote).await,
        }
    }

    async fn execute_local(job_args: JobArgs, local: LocalEntrypoint) -> Result<(), anyhow::Error> {
        let cwd = PathBuf::from(&local.cwd);
        let exec_cmd = local.exec;
        let job_args_json = serde_json::to_string(&job_args)?;

        let _ = tokio::task::spawn_blocking(move || {
            let shell = if cfg!(windows) { "cmd" } else { "sh" };
            let shell_flag = if cfg!(windows) { "/C" } else { "-c" };
            let full_cmd = format!("{} --job-args '{}'", exec_cmd, job_args_json);

            let output = Command::new(shell)
                .arg(shell_flag)
                .arg(&full_cmd)
                .current_dir(&cwd)
                .output();

            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!(
                        "\n--- stdout ---\n{}\n--- end stdout ---\n--- stderr ---\n{}\n--- end stderr ---",
                        stdout, stderr
                    );
                    if !output.status.success() {
                        eprintln!(
                            "Job '{}' failed with exit code {:?}",
                            job_args.job_id,
                            output.status.code()
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Failed to execute local entrypoint: {}", e);
                }
            }
        });
        Ok(())
    }

    async fn execute_remote(
        job_args: JobArgs,
        remote: RemoteEntrypoint,
    ) -> Result<(), anyhow::Error> {
        let url = remote.url;
        let headers = remote.headers;
        let job_args_json = serde_json::to_string(&job_args)?;

        let _ = tokio::spawn(async move {
            let client = reqwest::Client::new();
            let mut request = client.post(&url).body(job_args_json);

            for (key, value) in &headers {
                request = request.header(key, value);
            }

            match request.send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        eprintln!(
                            "Remote entrypoint returned status {}: {}",
                            response.status(),
                            response.text().await.unwrap_or_default()
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Failed to call remote entrypoint at {}: {}", url, e);
                }
            }
        });
        Ok(())
    }
}

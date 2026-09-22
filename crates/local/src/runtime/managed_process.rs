use std::io;
use std::process::ExitStatus;
#[cfg(unix)]
use std::time::Duration;

use process_wrap::tokio::{ChildWrapper, CommandWrap, KillOnDrop};
use tokio::process::Command;

#[cfg(windows)]
use process_wrap::tokio::JobObject;
#[cfg(unix)]
use process_wrap::tokio::ProcessGroup;

#[cfg(unix)]
const TERMINATION_GRACE_PERIOD: Duration = Duration::from_secs(2);

pub struct ManagedProcess {
    child: Box<dyn ChildWrapper>,
}

impl ManagedProcess {
    pub fn spawn(command: Command) -> io::Result<Self> {
        let mut command = CommandWrap::from(command);
        command.wrap(KillOnDrop);

        #[cfg(unix)]
        command.wrap(ProcessGroup::leader());
        #[cfg(windows)]
        command.wrap(JobObject);

        let child = command.spawn()?;
        Ok(Self { child })
    }

    pub async fn wait(&mut self) -> io::Result<ExitStatus> {
        self.child.wait().await
    }

    #[cfg(unix)]
    pub async fn kill(&mut self) -> io::Result<()> {
        if let Err(error) = self.child.signal(libc::SIGKILL) {
            if self.child.try_wait()?.is_none() {
                return Err(error);
            }
            return Ok(());
        }
        self.child.wait().await?;
        Ok(())
    }

    #[cfg(windows)]
    pub async fn kill(&mut self) -> io::Result<()> {
        self.child.kill().await
    }

    #[cfg(unix)]
    pub async fn terminate(&mut self) -> io::Result<()> {
        if let Err(error) = self.child.signal(libc::SIGTERM) {
            if self.child.try_wait()?.is_none() {
                return Err(error);
            }
            return Ok(());
        }

        tokio::time::sleep(TERMINATION_GRACE_PERIOD).await;
        if let Err(error) = self.child.start_kill() {
            if self.child.try_wait()?.is_none() {
                return Err(error);
            }
        }
        self.child.wait().await?;
        Ok(())
    }

    #[cfg(windows)]
    pub async fn terminate(&mut self) -> io::Result<()> {
        self.child.kill().await
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn terminates_a_process_tree_that_ignores_sigterm() {
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg("trap '' TERM; sh -c \"trap '' TERM; sleep 30\" & wait");
        let mut process = ManagedProcess::spawn(command).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        tokio::time::timeout(Duration::from_secs(5), process.terminate())
            .await
            .expect("process-tree termination timed out")
            .expect("process-tree termination failed");
    }
}

use std::{
    fs::{self, File},
    io::{BufReader, Read},
    path::PathBuf,
};

use tokio::fs::File as TokioFile;

use crate::models::JobRunId;

pub struct WorkerLog {
    job_run_id: JobRunId,
    cwd: Option<PathBuf>,
}

impl WorkerLog {
    pub fn new(job_run_id: JobRunId) -> Self {
        Self {
            job_run_id,
            cwd: None,
        }
    }

    pub fn in_directory(job_run_id: JobRunId, cwd: impl Into<PathBuf>) -> Self {
        Self {
            job_run_id,
            cwd: Some(cwd.into()),
        }
    }

    fn log_file_path(&self) -> PathBuf {
        // TODO: I think the engine will just read the log stream with no writing to disk
        // Then the client/python lib will be in charge of persisting the log to a file and storing that in
        // the user-configured store.
        // So, there is definitely a refactor brewing in this are of the architecture.
        let filename = format!("{}.log", self.job_run_id);
        match &self.cwd {
            Some(cwd) => cwd.join(&filename),
            None => PathBuf::from(filename),
        }
    }

    pub async fn get_write_file(&self) -> std::io::Result<TokioFile> {
        tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(self.log_file_path())
            .await
    }

    pub async fn get_read_file(&self) -> std::io::Result<TokioFile> {
        tokio::fs::OpenOptions::new()
            .read(true)
            .open(self.log_file_path())
            .await
    }

    fn get_read_file_sync(&self) -> std::io::Result<File> {
        fs::OpenOptions::new().read(true).open(self.log_file_path())
    }
}

pub struct WorkerLogReader {
    reader: BufReader<File>,
    contents: Vec<u8>,
}

impl WorkerLogReader {
    /// Opens a log and loads all content already present in the file.
    pub async fn new(job_run_id: JobRunId) -> std::io::Result<Self> {
        Self::new_sync(job_run_id)
    }

    /// Opens a log without depending on an async runtime.
    pub fn new_sync(job_run_id: JobRunId) -> std::io::Result<Self> {
        Self::from_log(WorkerLog::new(job_run_id))
    }

    /// Opens a log from a specific job entrypoint working directory.
    pub fn new_sync_in(job_run_id: JobRunId, cwd: impl Into<PathBuf>) -> std::io::Result<Self> {
        Self::from_log(WorkerLog::in_directory(job_run_id, cwd))
    }

    fn from_log(log: WorkerLog) -> std::io::Result<Self> {
        let file = log.get_read_file_sync()?;
        let mut reader = Self {
            reader: BufReader::new(file),
            contents: Vec::new(),
        };
        reader.refresh_sync()?;

        Ok(reader)
    }

    pub fn contents(&self) -> &[u8] {
        &self.contents
    }

    /// Appends bytes written since the previous refresh to the retained contents.
    pub async fn refresh(&mut self) -> std::io::Result<()> {
        self.refresh_sync()
    }

    /// Appends bytes written since the previous refresh to the retained contents.
    pub fn refresh_sync(&mut self) -> std::io::Result<()> {
        self.reader.read_to_end(&mut self.contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use tokio::io::AsyncWriteExt;

    use super::{WorkerLog, WorkerLogReader};
    use crate::models::JobRunId;

    #[tokio::test]
    async fn reader_loads_existing_content_before_watching_for_updates() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock should be after the Unix epoch")
            .as_nanos();
        let job_run_id = JobRunId::try_from(format!(
            "worker-log-reader-test-{}-{unique}",
            std::process::id()
        ))
        .expect("test job run ID should be valid");
        let log_path = WorkerLog::new(job_run_id.clone()).log_file_path();
        tokio::fs::write(&log_path, b"existing line\n")
            .await
            .expect("test log should be created");

        let mut reader = WorkerLogReader::new(job_run_id.clone())
            .await
            .expect("test log should open");
        assert_eq!(reader.contents(), b"existing line\n");

        let mut writer = WorkerLog::new(job_run_id)
            .get_write_file()
            .await
            .expect("test log writer should open");
        writer
            .write_all(b"new line\n")
            .await
            .expect("new log line should be written");
        writer
            .flush()
            .await
            .expect("new log line should be flushed");
        reader
            .refresh()
            .await
            .expect("reader should load appended content");
        assert_eq!(reader.contents(), b"existing line\nnew line\n");

        tokio::fs::remove_file(log_path)
            .await
            .expect("test log should be removed");
    }

    #[test]
    fn reader_loads_log_from_entrypoint_working_directory() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock should be after the Unix epoch")
            .as_nanos();
        let job_run_id = JobRunId::try_from(format!(
            "worker-log-reader-cwd-test-{}-{unique}",
            std::process::id()
        ))
        .expect("test job run ID should be valid");
        let cwd = std::env::temp_dir().join(format!("zygo-worker-log-cwd-{unique}"));
        std::fs::create_dir_all(&cwd).expect("test working directory should be created");

        let log_path = WorkerLog::in_directory(job_run_id.clone(), cwd.clone()).log_file_path();
        std::fs::write(&log_path, b"entrypoint line\n").expect("test log should be created");

        let reader = WorkerLogReader::new_sync_in(job_run_id, cwd.clone())
            .expect("test log should open from its working directory");
        assert_eq!(reader.contents(), b"entrypoint line\n");

        std::fs::remove_dir_all(cwd).expect("test working directory should be removed");
    }
}

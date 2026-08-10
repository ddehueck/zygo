use std::path::PathBuf;

use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, BufReader},
};

use crate::models::JobRunId;

pub struct WorkerLog {
    job_run_id: JobRunId,
}

impl WorkerLog {
    pub fn new(job_run_id: JobRunId) -> Self {
        Self { job_run_id }
    }

    fn log_file_path(&self) -> PathBuf {
        // TODO: We'll need the workflow metadata to include the store location
        // for now, we'll ignore that and use a relative path
        PathBuf::from(format!("{}.log", self.job_run_id))
    }

    pub async fn get_write_file(&self) -> std::io::Result<File> {
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(self.log_file_path())
            .await
    }

    pub async fn get_read_file(&self) -> std::io::Result<File> {
        fs::OpenOptions::new()
            .read(true)
            .open(self.log_file_path())
            .await
    }
}

pub struct WorkerLogReader {
    reader: BufReader<File>,
    contents: Vec<u8>,
}

impl WorkerLogReader {
    /// Opens a log and loads all content already present in the file.
    pub async fn new(job_run_id: JobRunId) -> std::io::Result<Self> {
        let file = WorkerLog::new(job_run_id).get_read_file().await?;
        let mut reader = Self {
            reader: BufReader::new(file),
            contents: Vec::new(),
        };
        reader.refresh().await?;

        Ok(reader)
    }

    pub fn contents(&self) -> &[u8] {
        &self.contents
    }

    /// Appends bytes written since the previous refresh to the retained contents.
    pub async fn refresh(&mut self) -> std::io::Result<()> {
        self.reader.read_to_end(&mut self.contents).await?;
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
}

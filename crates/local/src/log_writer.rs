use zygo_core::LogWriter;
use zygo_core::models::{JobRunSource, WorkflowRunId};

use crate::LogsRepository;

impl LogWriter for LogsRepository {
    async fn write_all(
        &self,
        workflow_run_id: &WorkflowRunId,
        source: &JobRunSource,
        bytes: &[u8],
    ) -> std::io::Result<()> {
        // The runner supplies complete lines, including the final unterminated line at EOF.
        // Preserve delimiters and replace invalid UTF-8 for the searchable TEXT column.
        let content = String::from_utf8_lossy(bytes);
        let lines = content.split_inclusive('\n').collect::<Vec<_>>();
        self.append(
            &workflow_run_id.to_string(),
            &source.job_run_id.to_string(),
            &source.job_id.to_string(),
            &lines,
        )
        .await
        .map_err(std::io::Error::other)
    }
}

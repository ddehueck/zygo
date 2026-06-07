use std::collections::HashMap;

use zygo_core::models::{JobRunId, SequenceId, Source, StreamItem, StreamRecord};

use super::{Invariant, InvariantOutcome};

/// Event sequence ids should be strictly increasing within each job run.
pub struct CheckOrderedRunEvents {
    records: Vec<StreamRecord>,
}

impl CheckOrderedRunEvents {
    pub fn new(records: Vec<StreamRecord>) -> Self {
        Self { records }
    }
}

impl Invariant for CheckOrderedRunEvents {
    fn name(&self) -> &str {
        "check_ordered_run_events"
    }

    fn check(&self) -> InvariantOutcome {
        let mut job_run_sequence_ids: HashMap<JobRunId, Vec<SequenceId>> = HashMap::new();

        for record in &self.records {
            if let StreamItem::Event(event) = &record.item {
                if let Source::JobRun(job_run_source) = &event.source {
                    job_run_sequence_ids
                        .entry(job_run_source.job_run_id.clone())
                        .or_default()
                        .push(record.id);
                }
            }
        }

        let mut violations = Vec::new();

        for (job_run_id, sequence_ids) in &job_run_sequence_ids {
            for window in sequence_ids.windows(2) {
                if window[0] >= window[1] {
                    violations.push(format!(
                        "Event keys are not ordered within job run {job_run_id:?}: {:?} >= {:?}",
                        window[0], window[1]
                    ));
                }
            }
        }

        if violations.is_empty() {
            InvariantOutcome::Passed
        } else {
            InvariantOutcome::Failed(violations)
        }
    }
}

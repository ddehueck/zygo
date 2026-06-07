use zygo_core::models::{Command, EventKind, StreamItem, StreamRecord};

use super::{Invariant, InvariantOutcome};

pub struct CheckReplayMatchesOriginal {
    original_records: Vec<StreamRecord>,
    replay_records: Vec<StreamRecord>,
}

impl CheckReplayMatchesOriginal {
    pub fn new(original_records: Vec<StreamRecord>, replay_records: Vec<StreamRecord>) -> Self {
        Self {
            original_records,
            replay_records,
        }
    }
}

impl Invariant for CheckReplayMatchesOriginal {
    fn name(&self) -> &str {
        "check_replay_matches_original"
    }

    fn check(&self) -> InvariantOutcome {
        let original_events = self
            .original_records
            .iter()
            .filter(|r| matches!(r.item, StreamItem::Event(_)))
            .collect::<Vec<_>>();

        let replay_events = self
            .replay_records
            .iter()
            .filter(|r| matches!(r.item, StreamItem::Event(_)))
            .collect::<Vec<_>>();

        if original_events.len() != replay_events.len() {
            return InvariantOutcome::Failed(vec![format!(
                "event mismatch on rerun\n\
                 original events:\n{}\n\
                 replay events:\n{}",
                record_summary_string(&self.original_records),
                record_summary_string(&self.replay_records),
            )]);
        }

        return InvariantOutcome::Passed;
    }
}

fn record_summary_string(records: &[StreamRecord]) -> String {
    let mut summary = String::new();
    for record in records {
        summary.push_str(&format_record_summary(record));
        summary.push('\n');
    }
    summary
}

fn format_record_summary(record: &StreamRecord) -> String {
    match &record.item {
        StreamItem::Event(event) => {
            format!("{} Event {}", record.id, event_kind_name(&event.kind))
        }
        StreamItem::Command(command) => {
            format!("{} Command {}", record.id, command_name(command))
        }
    }
}

fn event_kind_name(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::DataReferenceInserted(_) => "DataReferenceInserted",
        EventKind::ChannelItemInserted(_) => "ChannelItemInserted",
        EventKind::JobStarted(_) => "JobStarted",
        EventKind::JobSucceeded(_) => "JobSucceeded",
        EventKind::JobFailed(_) => "JobFailed",
    }
}

fn command_name(command: &Command) -> &'static str {
    match command {
        Command::RunJob(_) => "RunJob",
        Command::ReplayJob(_) => "ReplayJob",
        Command::CacheJobRunResult(_) => "CacheJobRunResult",
        Command::CacheJobEventSource(_) => "CacheJobEventSource",
        Command::SetJobRunStatus(_) => "SetJobRunStatus",
    }
}

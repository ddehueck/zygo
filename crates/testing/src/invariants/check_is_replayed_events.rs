use zygo_core::models::{Source, StreamItem, StreamRecord};

use super::{Invariant, InvariantOutcome};

pub struct CheckIsReplayedEvents {
    records: Vec<StreamRecord>,
}

impl CheckIsReplayedEvents {
    pub fn new(records: Vec<StreamRecord>) -> Self {
        Self { records }
    }
}

impl Invariant for CheckIsReplayedEvents {
    fn name(&self) -> &str {
        "check_is_replayed_events"
    }

    fn check(&self) -> InvariantOutcome {
        let mut violations = Vec::new();
        for record in &self.records {
            if let StreamItem::Event(event) = &record.item {
                if let Source::Input(_) = &event.source {
                    continue;
                }

                if !event.is_replay {
                    violations.push(format!("event {:?} is not a replay", record.id));
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

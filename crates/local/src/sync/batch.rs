use std::collections::HashSet;

use super::schema::Delta;

pub struct DeltaBatch {
    pub max_change_id: i64,
    pub deltas: Vec<Delta>,
}

impl DeltaBatch {
    pub fn new(max_change_id: i64, deltas: Vec<Delta>) -> Self {
        Self {
            max_change_id,
            deltas,
        }
        .coalesce_latest()
    }

    /// Returns the last change for each entity and item ID in this batch.
    ///
    /// CDC rows are ordered oldest to newest. Walking them backwards means
    /// that the first row retained for an ID is the most recent one, while
    /// reversing the result restores the order of those retained changes.
    pub fn coalesce_latest(mut self) -> Self {
        let mut seen = HashSet::new();
        let mut has_resync = false;
        let mut deltas = self
            .deltas
            .iter()
            .rev()
            .filter_map(|delta| match delta {
                Delta::Resync if !has_resync => {
                    has_resync = true;
                    Some(delta.clone())
                }
                Delta::Resync => None,
                Delta::Delete { entity, id } | Delta::Upsert { entity, id, .. }
                    if seen.insert((*entity, id.as_str())) =>
                {
                    Some(delta.clone())
                }
                Delta::Delete { .. } | Delta::Upsert { .. } => None,
            })
            .collect::<Vec<_>>();

        deltas.reverse();
        self.deltas = deltas;
        self
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::DeltaBatch;
    use crate::sync::schema::{Delta, SyncEntity};

    fn upsert(entity: SyncEntity, id: &str, status: &str) -> Delta {
        Delta::Upsert {
            entity,
            id: id.to_owned(),
            data: json!({"id": id, "status": status}),
        }
    }

    #[test]
    fn keeps_only_the_latest_change_for_each_item() {
        let batch = DeltaBatch::new(
            4,
            vec![
                upsert(SyncEntity::WorkflowRun, "run-1", "started"),
                upsert(SyncEntity::WorkflowRun, "run-2", "started"),
                Delta::Delete {
                    entity: SyncEntity::WorkflowRun,
                    id: "run-1".to_owned(),
                },
                upsert(SyncEntity::WorkflowRun, "run-2", "completed"),
            ],
        );

        assert_eq!(batch.max_change_id, 4);
        assert_eq!(
            batch.deltas,
            vec![
                Delta::Delete {
                    entity: SyncEntity::WorkflowRun,
                    id: "run-1".to_owned(),
                },
                upsert(SyncEntity::WorkflowRun, "run-2", "completed"),
            ]
        );
    }
}

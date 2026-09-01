use turso::Value as SqlValue;

use crate::{CdcChangeType, CdcRow};

use super::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyncEntity {
    WorkflowRunSummary,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Delta {
    Resync,
    Delete {
        entity: SyncEntity,
        id: String,
    },
    Upsert {
        entity: SyncEntity,
        id: String,
        data: serde_json::Value,
    },
}

impl TryFrom<CdcRow> for Delta {
    type Error = Error;

    fn try_from(row: CdcRow) -> Result<Self> {
        let entity = match row.table_name.as_str() {
            "workflow_run_summary" => SyncEntity::WorkflowRunSummary,
            table_name => return Err(Error::UnsupportedTable(table_name.to_owned())),
        };

        let id = cdc_id(row.id);

        match row.change_type {
            CdcChangeType::Insert | CdcChangeType::Update => {
                let data = row.after.ok_or_else(|| Error::MissingAfter {
                    change_id: row.change_id,
                    change_type: row.change_type.clone(),
                    table_name: row.table_name.clone(),
                })?;

                Ok(Delta::Upsert { entity, id, data })
            }
            CdcChangeType::Delete => Ok(Delta::Delete { entity, id }),
        }
    }
}

fn cdc_id(id: SqlValue) -> String {
    match id {
        SqlValue::Text(id) => id,
        SqlValue::Integer(id) => id.to_string(),
        SqlValue::Real(id) => id.to_string(),
        SqlValue::Null => "null".to_owned(),
        SqlValue::Blob(id) => String::from_utf8_lossy(&id).into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use turso::Value as SqlValue;

    use crate::{
        CdcChangeType, CdcRow,
        sync::{
            error::Error,
            schema::{Delta, SyncEntity},
        },
    };

    fn cdc_row(
        table_name: &str,
        change_type: CdcChangeType,
        after: Option<serde_json::Value>,
    ) -> CdcRow {
        CdcRow {
            change_id: 42,
            change_time: 0,
            change_txn_id: 1,
            change_type,
            table_name: table_name.to_owned(),
            id: SqlValue::Integer(1),
            after,
        }
    }

    #[test]
    fn converts_supported_table_to_upsert() {
        let delta = Delta::try_from(cdc_row(
            "workflow_run_summary",
            CdcChangeType::Update,
            Some(json!({"workflow_run_id": "run-1"})),
        ))
        .expect("supported CDC row should convert");

        match delta {
            Delta::Upsert { entity, data, .. } => {
                assert!(matches!(entity, SyncEntity::WorkflowRunSummary));
                assert_eq!(data, json!({"workflow_run_id": "run-1"}));
            }
            _ => panic!("expected an upsert delta"),
        }
    }

    #[test]
    fn rejects_workflow_runs_as_an_unsupported_table() {
        let result = Delta::try_from(cdc_row(
            "workflow_runs",
            CdcChangeType::Insert,
            Some(json!({"id": "run-1"})),
        ));

        assert!(matches!(
            result,
            Err(Error::UnsupportedTable(table_name)) if table_name == "workflow_runs"
        ));
    }

    #[test]
    fn rejects_unsupported_table() {
        let result = Delta::try_from(cdc_row(
            "kv",
            CdcChangeType::Update,
            Some(json!({"key": "k"})),
        ));

        assert!(matches!(
            result,
            Err(Error::UnsupportedTable(table_name)) if table_name == "kv"
        ));
    }
}

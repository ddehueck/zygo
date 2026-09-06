use super::{Error, Result};
use crate::{CdcChangeType, CdcRow};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyncEntity {
    WorkflowRun,
    JobRun,
    Tag,
    DataReference,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Delta {
    Insert {
        change_id: i64,
        entity: SyncEntity,
        data: serde_json::Value,
    },
    Update {
        change_id: i64,
        entity: SyncEntity,
        data: serde_json::Value,
    },
    Delete {
        change_id: i64,
        entity: SyncEntity,
        id: i64,
    },
}

impl TryFrom<CdcRow> for Delta {
    type Error = Error;

    fn try_from(row: CdcRow) -> Result<Self> {
        let entity = match row.table_name.as_str() {
            "workflow_runs" => SyncEntity::WorkflowRun,
            "job_runs" => SyncEntity::JobRun,
            "tags" => SyncEntity::Tag,
            "data_references" => SyncEntity::DataReference,
            table_name => return Err(Error::UnsupportedTable(table_name.to_owned())),
        };

        match row.change_type {
            CdcChangeType::Insert => {
                let data = row.after.ok_or_else(|| Error::MissingAfter {
                    change_id: row.change_id,
                    change_type: row.change_type.clone(),
                    table_name: row.table_name.clone(),
                })?;

                Ok(Delta::Insert {
                    change_id: row.change_id,
                    entity,
                    data,
                })
            }
            CdcChangeType::Update => {
                let data = row.after.ok_or_else(|| Error::MissingAfter {
                    change_id: row.change_id,
                    change_type: row.change_type.clone(),
                    table_name: row.table_name.clone(),
                })?;

                Ok(Delta::Update {
                    change_id: row.change_id,
                    entity,
                    data,
                })
            }
            CdcChangeType::Delete => {
                let turso::Value::Integer(id) = row.id else {
                    return Err(Error::InvalidRowId {
                        change_id: row.change_id,
                        table_name: row.table_name,
                    });
                };

                Ok(Delta::Delete {
                    change_id: row.change_id,
                    entity,
                    id,
                })
            }
        }
    }
}

use super::{Error, Result};
use crate::{CdcChangeType, CdcRow, DataReferenceModel, JobRunModel, TagModel, WorkflowRunModel};

#[derive(Debug, Clone, PartialEq)]
pub enum RowChange<T> {
    Insert { row: T },
    Update { row: T },
    Delete { id: i64 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Delta {
    WorkflowRun {
        change_id: i64,
        change: RowChange<WorkflowRunModel>,
    },
    JobRun {
        change_id: i64,
        change: RowChange<JobRunModel>,
    },
    Tag {
        change_id: i64,
        change: RowChange<TagModel>,
    },
    DataReference {
        change_id: i64,
        change: RowChange<DataReferenceModel>,
    },
}

impl<T> RowChange<T> {
    fn from_cdc_row(
        row: CdcRow,
        decode: impl FnOnce(serde_json::Value) -> crate::DbResult<T>,
    ) -> Result<Self> {
        match row.change_type {
            CdcChangeType::Insert | CdcChangeType::Update => {
                let after = row.after.ok_or_else(|| Error::MissingAfter {
                    change_id: row.change_id,
                    change_type: row.change_type.clone(),
                    table_name: row.table_name.clone(),
                })?;
                // Decode the captured after-image, never the current database state.
                let data = decode(after).map_err(|source| Error::InvalidAfter {
                    change_id: row.change_id,
                    table_name: row.table_name,
                    source,
                })?;

                match row.change_type {
                    CdcChangeType::Insert => Ok(Self::Insert { row: data }),
                    _ => Ok(Self::Update { row: data }),
                }
            }
            CdcChangeType::Delete => {
                let turso::Value::Integer(id) = row.id else {
                    return Err(Error::InvalidRowId {
                        change_id: row.change_id,
                        table_name: row.table_name,
                    });
                };

                Ok(Self::Delete { id })
            }
        }
    }
}

impl TryFrom<CdcRow> for Delta {
    type Error = Error;

    fn try_from(row: CdcRow) -> Result<Self> {
        let change_id = row.change_id;
        match row.table_name.as_str() {
            "workflow_runs" => Ok(Self::WorkflowRun {
                change_id,
                change: RowChange::from_cdc_row(row, WorkflowRunModel::from_sql_value)?,
            }),
            "job_runs" => Ok(Self::JobRun {
                change_id,
                change: RowChange::from_cdc_row(row, JobRunModel::from_sql_value)?,
            }),
            "tags" => Ok(Self::Tag {
                change_id,
                change: RowChange::from_cdc_row(row, TagModel::from_sql_value)?,
            }),
            "data_references" => Ok(Self::DataReference {
                change_id,
                change: RowChange::from_cdc_row(row, DataReferenceModel::from_sql_value)?,
            }),
            table_name => Err(Error::UnsupportedTable(table_name.to_owned())),
        }
    }
}

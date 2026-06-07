use crate::db::models::EventRow;
use crate::models::data_reference::DataReference;
use crate::models::event::{
    ChannelItemInsertedData, DataReferenceInsertedData, Event, EventKind, InputSource,
    JobFailedData, JobRequestedData, JobRunSource, JobStartedData, JobSucceededData, Source,
};
use crate::models::{ChannelId, DomainError, JobId, RunId, WorkflowVersionId};

use super::datetime::{naive_datetime_to_system_time, system_time_to_naive_datetime};

/// Reconstruct the Source from the EventRow's source columns.
fn source_from_row(row: &EventRow) -> Result<Source, DomainError> {
    match row.source_type.as_str() {
        "input" => Ok(Source::Input(InputSource {
            workflow_run_id: RunId::try_from(row.workflow_run_id.clone())?,
        })),
        "job_run" => {
            let job_id = row
                .source_job_id
                .as_ref()
                .ok_or_else(|| DomainError::missing("source_job_id"))?;
            let job_run_id = row
                .source_job_run_id
                .as_ref()
                .ok_or_else(|| DomainError::missing("source_job_run_id"))?;
            Ok(Source::JobRun(JobRunSource {
                job_id: JobId::try_from(job_id.clone())?,
                job_run_id: job_run_id.clone(),
                workflow_run_id: RunId::try_from(row.workflow_run_id.clone())?,
            }))
        }
        other => Err(DomainError::invalid(
            "source_type",
            &format!("unknown source type: {other}"),
        )),
    }
}

/// Convert an EventRow + its associated DataReference into a domain Event.
///
/// `data_reference` must be `Some` for event kinds that carry data
/// (data_reference_inserted, channel_item_inserted, job_requested).
/// It is loaded separately by the EventStore from the data_references table.
pub fn event_from_row(
    row: EventRow,
    data_reference: Option<DataReference>,
) -> Result<Event, DomainError> {
    let source = source_from_row(&row)?;

    let kind = match row.kind.as_str() {
        "data_reference_inserted" => {
            let dr = data_reference.ok_or_else(|| DomainError::missing("data_reference"))?;
            EventKind::DataReferenceInserted(DataReferenceInsertedData { data_reference: dr })
        }
        "channel_item_inserted" => {
            let dr = data_reference.ok_or_else(|| DomainError::missing("data_reference"))?;
            let channel_id = row
                .inserted_channel_id
                .ok_or_else(|| DomainError::missing("inserted_channel_id"))?;
            EventKind::ChannelItemInserted(ChannelItemInsertedData {
                channel_id: ChannelId::try_from(channel_id)?,
                data_reference: dr,
            })
        }
        "job_requested" => {
            let dr = data_reference.ok_or_else(|| DomainError::missing("data_reference"))?;
            let job_id = row.job_id.ok_or_else(|| DomainError::missing("job_id"))?;
            EventKind::JobRequested(JobRequestedData {
                job_id: JobId::try_from(job_id)?,
                data_reference: dr,
            })
        }
        "job_started" => EventKind::JobStarted(JobStartedData {}),
        "job_succeeded" => EventKind::JobSucceeded(JobSucceededData {}),
        "job_failed" => EventKind::JobFailed(JobFailedData {}),
        other => {
            return Err(DomainError::invalid(
                "kind",
                &format!("unknown event kind: {other}"),
            ));
        }
    };

    Ok(Event {
        id: row.id,
        is_replay: row.is_replay != 0,
        timestamp: naive_datetime_to_system_time(row.timestamp),
        sequence_number: Some(row.sequence_number),
        kind,
        source,
        workflow_version_id: Some(WorkflowVersionId::try_from(row.workflow_version_id)?),
    })
}

/// Convert an Event to an EventRow for persistence.
///
/// Note: `data_reference_id` is set to `None` here because the domain Event
/// carries a full DataReference (no id). The EventStore resolves the id
/// by looking up (uri, etag) in the data_references table before inserting.
impl EventRow {
    pub fn from_event(event: &Event, workflow_version_id_override: &str) -> Self {
        let workflow_version_id = event
            .workflow_version_id
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_else(|| workflow_version_id_override.to_string());
        let (source_type, source_job_id, source_job_run_id, workflow_run_id) = match &event.source {
            Source::Input(input) => (
                "input".to_string(),
                None,
                None,
                input.workflow_run_id.to_string(),
            ),
            Source::JobRun(job_run) => (
                "job_run".to_string(),
                Some(job_run.job_id.to_string()),
                Some(job_run.job_run_id.clone()),
                job_run.workflow_run_id.to_string(),
            ),
        };

        let (kind, job_id, inserted_channel_id) = match &event.kind {
            EventKind::DataReferenceInserted(_) => {
                ("data_reference_inserted".to_string(), None, None)
            }
            EventKind::ChannelItemInserted(data) => (
                "channel_item_inserted".to_string(),
                None,
                Some(data.channel_id.to_string()),
            ),
            EventKind::JobRequested(data) => (
                "job_requested".to_string(),
                Some(data.job_id.to_string()),
                None,
            ),
            EventKind::JobStarted(_) => ("job_started".to_string(), None, None),
            EventKind::JobSucceeded(_) => ("job_succeeded".to_string(), None, None),
            EventKind::JobFailed(_) => ("job_failed".to_string(), None, None),
        };

        Self {
            id: event.id.clone(),
            workflow_version_id,
            is_replay: if event.is_replay { 1 } else { 0 },
            timestamp: system_time_to_naive_datetime(event.timestamp),
            workflow_run_id,
            sequence_number: event
                .sequence_number
                .expect("sequence_number must be assigned before persisting event"),
            source_type,
            source_job_id,
            source_job_run_id,
            kind,
            job_id,
            inserted_channel_id,
            // data_reference_id is resolved by EventStore before INSERT
            data_reference_id: None,
        }
    }
}

/// Extract the (uri, etag) from an EventKind's DataReference, if present.
pub fn extract_data_reference_key(kind: &EventKind) -> Option<(&str, &str)> {
    match kind {
        EventKind::DataReferenceInserted(data) => {
            Some((&data.data_reference.uri, &data.data_reference.etag))
        }
        EventKind::ChannelItemInserted(data) => {
            Some((&data.data_reference.uri, &data.data_reference.etag))
        }
        EventKind::JobRequested(data) => {
            Some((&data.data_reference.uri, &data.data_reference.etag))
        }
        EventKind::JobStarted(_) | EventKind::JobSucceeded(_) | EventKind::JobFailed(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn event_row_job_requested_to_domain() {
        let row = EventRow {
            id: "evt_123".to_string(),
            workflow_version_id: "wv_456".to_string(),
            is_replay: 0,
            timestamp: chrono::Utc::now().naive_utc(),
            workflow_run_id: "run_789".to_string(),
            sequence_number: 1,
            source_type: "input".to_string(),
            source_job_id: None,
            source_job_run_id: None,
            kind: "job_requested".to_string(),
            job_id: Some("job_001".to_string()),
            inserted_channel_id: None,
            data_reference_id: Some("dr_001".to_string()),
        };

        let dr = DataReference {
            uri: "s3://bucket/key".to_string(),
            etag: "abc123".to_string(),
            content_type: Some("application/json".to_string()),
            size_bytes: Some(1024),
        };

        let event = event_from_row(row, Some(dr)).unwrap();
        assert_eq!(event.id, "evt_123");
        match &event.source {
            Source::Input(input) => assert_eq!(input.workflow_run_id.as_ref(), "run_789"),
            _ => panic!("Expected Input source"),
        }
        match &event.kind {
            EventKind::JobRequested(data) => {
                assert_eq!(data.job_id.as_ref(), "job_001");
                assert_eq!(data.data_reference.uri, "s3://bucket/key");
                assert_eq!(data.data_reference.etag, "abc123");
                assert_eq!(
                    data.data_reference.content_type.as_deref(),
                    Some("application/json")
                );
                assert_eq!(data.data_reference.size_bytes, Some(1024));
            }
            _ => panic!("Expected JobRequested"),
        }
    }

    #[test]
    fn event_row_job_started_to_domain() {
        let row = EventRow {
            id: "evt_124".to_string(),
            workflow_version_id: "wv_456".to_string(),
            is_replay: 1,
            timestamp: chrono::Utc::now().naive_utc(),
            workflow_run_id: "run_789".to_string(),
            sequence_number: 2,
            source_type: "job_run".to_string(),
            source_job_id: Some("job_001".to_string()),
            source_job_run_id: Some("jr_001".to_string()),
            kind: "job_started".to_string(),
            job_id: None,
            inserted_channel_id: None,
            data_reference_id: None,
        };

        let event = event_from_row(row, None).unwrap();
        assert!(event.is_replay);
        match &event.source {
            Source::JobRun(jr) => {
                assert_eq!(jr.job_id.as_ref(), "job_001");
                assert_eq!(jr.job_run_id, "jr_001");
            }
            _ => panic!("Expected JobRun source"),
        }
        assert!(matches!(event.kind, EventKind::JobStarted(_)));
    }

    #[test]
    fn event_row_channel_item_inserted_to_domain() {
        let row = EventRow {
            id: "evt_125".to_string(),
            workflow_version_id: "wv_456".to_string(),
            is_replay: 0,
            timestamp: chrono::Utc::now().naive_utc(),
            workflow_run_id: "run_789".to_string(),
            sequence_number: 3,
            source_type: "input".to_string(),
            source_job_id: None,
            source_job_run_id: None,
            kind: "channel_item_inserted".to_string(),
            job_id: None,
            inserted_channel_id: Some("my-channel".to_string()),
            data_reference_id: Some("dr_002".to_string()),
        };

        let dr = DataReference {
            uri: "s3://bucket/item".to_string(),
            etag: "etag_002".to_string(),
            content_type: None,
            size_bytes: None,
        };

        let event = event_from_row(row, Some(dr)).unwrap();
        match &event.kind {
            EventKind::ChannelItemInserted(data) => {
                assert_eq!(data.channel_id.as_ref(), "my-channel");
                assert_eq!(data.data_reference.uri, "s3://bucket/item");
                assert_eq!(data.data_reference.etag, "etag_002");
            }
            _ => panic!("Expected ChannelItemInserted"),
        }
    }

    #[test]
    fn event_domain_to_row() {
        let event = Event {
            id: "evt_999".to_string(),
            is_replay: true,
            timestamp: SystemTime::now(),
            sequence_number: Some(5),
            kind: EventKind::JobSucceeded(JobSucceededData {}),
            source: Source::JobRun(JobRunSource {
                job_id: JobId::try_from("job_333".to_string()).unwrap(),
                job_run_id: "jr_444".to_string(),
                workflow_run_id: RunId::try_from("run_222".to_string()).unwrap(),
            }),
            workflow_version_id: None,
        };

        let row = EventRow::from_event(&event, "wv_111");
        assert_eq!(row.id, "evt_999");
        assert_eq!(row.kind, "job_succeeded");
        assert_eq!(row.is_replay, 1);
        assert_eq!(row.source_type, "job_run");
        assert_eq!(row.source_job_id, Some("job_333".to_string()));
        assert_eq!(row.source_job_run_id, Some("jr_444".to_string()));
        assert_eq!(row.workflow_run_id, "run_222");
        assert_eq!(row.workflow_version_id, "wv_111");
        // data_reference_id is None (resolved by EventStore)
        assert!(row.data_reference_id.is_none());
    }
}

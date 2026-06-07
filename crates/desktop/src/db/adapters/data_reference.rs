use uuid::Uuid;

use crate::db::models::DataReferenceRow;
use crate::models::data_reference::DataReference;
use crate::models::DomainError;

impl TryFrom<DataReferenceRow> for DataReference {
    type Error = DomainError;

    fn try_from(row: DataReferenceRow) -> Result<Self, Self::Error> {
        Ok(Self {
            uri: row.uri,
            etag: row.etag,
            content_type: row.content_type,
            size_bytes: row.size_bytes.map(|v| v as u64),
        })
    }
}

impl From<DataReference> for DataReferenceRow {
    fn from(data_ref: DataReference) -> Self {
        // Generate a UUID v7 for the DB row's id since the domain model no longer carries one
        let id = Uuid::now_v7().to_string();
        Self {
            id,
            uri: data_ref.uri,
            etag: data_ref.etag,
            content_type: data_ref.content_type,
            size_bytes: data_ref.size_bytes.map(|v| v as i64),
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_reference_row_to_domain() {
        let row = DataReferenceRow {
            id: "dr_123".to_string(),
            uri: "s3://bucket/key".to_string(),
            etag: "abc123".to_string(),
            content_type: Some("application/json".to_string()),
            size_bytes: Some(1024),
            created_at: chrono::Utc::now().naive_utc(),
        };

        let dr = DataReference::try_from(row).unwrap();
        assert_eq!(dr.uri, "s3://bucket/key");
        assert_eq!(dr.etag, "abc123");
        assert_eq!(dr.content_type.as_deref(), Some("application/json"));
        assert_eq!(dr.size_bytes, Some(1024));
    }

    #[test]
    fn data_reference_domain_to_row() {
        let dr = DataReference {
            uri: "gs://bucket/obj".to_string(),
            etag: "def456".to_string(),
            content_type: None,
            size_bytes: None,
        };

        let row = DataReferenceRow::from(dr);
        assert!(!row.id.is_empty()); // UUID v7 is generated
        assert_eq!(row.uri, "gs://bucket/obj");
        assert_eq!(row.etag, "def456");
        assert!(row.content_type.is_none());
        assert!(row.size_bytes.is_none());
    }
}

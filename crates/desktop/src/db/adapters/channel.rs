use crate::db::models::ChannelRow;
use crate::models::{Channel, ChannelId, ChannelName, DomainError, WorkflowVersionId};

impl TryFrom<ChannelRow> for Channel {
    type Error = DomainError;

    fn try_from(row: ChannelRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ChannelId::try_from(row.id)?,
            name: ChannelName::try_from(row.name)?,
            workflow_version_id: WorkflowVersionId::try_from(row.workflow_version_id)?,
        })
    }
}

impl From<Channel> for ChannelRow {
    fn from(channel: Channel) -> Self {
        Self {
            id: channel.id.into(),
            name: channel.name.into(),
            workflow_version_id: channel.workflow_version_id.into(),
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_row_to_domain() {
        let row = ChannelRow {
            id: "ch_123".to_string(),
            name: "my-channel".to_string(),
            workflow_version_id: "wv_456".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
        };

        let channel = Channel::try_from(row).unwrap();
        assert_eq!(channel.id.as_ref(), "ch_123");
        assert_eq!(channel.name.as_ref(), "my-channel");
        assert_eq!(channel.workflow_version_id.as_ref(), "wv_456");
    }

    #[test]
    fn channel_domain_to_row() {
        let channel = Channel {
            id: ChannelId::try_from("ch_789".to_string()).unwrap(),
            name: ChannelName::try_from("test-channel".to_string()).unwrap(),
            workflow_version_id: WorkflowVersionId::try_from("wv_012".to_string()).unwrap(),
        };

        let row = ChannelRow::from(channel);
        assert_eq!(row.id, "ch_789");
        assert_eq!(row.name, "test-channel");
        assert_eq!(row.workflow_version_id, "wv_012");
    }
}


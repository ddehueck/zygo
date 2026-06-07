use serde::{Deserialize, Serialize};

use crate::models::ids::{ChannelId, ChannelName};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: ChannelId,
    pub name: ChannelName,
}

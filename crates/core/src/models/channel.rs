use serde::{Deserialize, Serialize};

use crate::models::ids::ChannelId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: ChannelId,
}

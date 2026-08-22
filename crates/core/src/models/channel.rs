use serde::{Deserialize, Serialize};

use crate::models::{ChannelId, FileExtension};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: ChannelId,
    pub accepted_file_extensions: Vec<FileExtension>,
}

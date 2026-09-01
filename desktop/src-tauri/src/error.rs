use serde::Serialize;
use specta::Type;

/// The error contract exposed by Tauri commands to the frontend.
#[derive(Debug, Serialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CommandError {
    InvalidInput { field: String, message: String },
    Internal { code: String, message: String },
}

impl CommandError {
    pub fn invalid_input(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidInput {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn internal(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Internal {
            code: code.into(),
            message: message.into(),
        }
    }
}

pub type CommandResult<T> = std::result::Result<T, CommandError>;

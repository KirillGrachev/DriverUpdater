use serde::Serialize;

/// Serializable error returned by commands that need a stable UI code.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<String> for CommandError {
    fn from(message: String) -> Self {
        Self::from_message(message)
    }
}

impl CommandError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }

    pub fn from_message(message: String) -> Self {
        let code = if message.starts_with("E_RESTORE_POINT") {
            "restorePoint"
        } else if message == "E_PACKAGE_CANCELLED" {
            "packageCancelled"
        } else {
            "internal"
        };
        Self::new(code, message)
    }
}

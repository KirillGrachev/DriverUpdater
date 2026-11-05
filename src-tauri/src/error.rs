use serde::Serialize;

/// Serializable error returned by commands that need a stable UI code.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<CommandError> for String {
    fn from(error: CommandError) -> Self {
        error.message
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_a_stable_error_code() {
        let value =
            serde_json::to_value(CommandError::new("restorePoint", "failed")).expect("json");
        assert_eq!(value["code"], "restorePoint");
        assert_eq!(value["message"], "failed");
    }

    #[test]
    fn classifies_legacy_error_messages() {
        assert_eq!(
            CommandError::from_message("E_RESTORE_POINT: disabled".to_string()).code,
            "restorePoint"
        );
        assert_eq!(
            CommandError::from_message("E_PACKAGE_CANCELLED".to_string()).code,
            "packageCancelled"
        );
    }
}

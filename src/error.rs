use crate::budget::BudgetDimension;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Error)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Error {
    #[error("invalid input ({code}): {message}")]
    InvalidInput { code: String, message: String },
    #[error("unsupported ({code}): {message}")]
    Unsupported { code: String, message: String },
    #[error(
        "budget exceeded for {dimension:?}: limit={limit}, consumed={consumed}, requested={requested}"
    )]
    BudgetExceeded {
        dimension: BudgetDimension,
        limit: u64,
        consumed: u64,
        requested: u64,
    },
    #[error("cancelled: {reason}")]
    Cancelled { reason: String },
    #[error("I/O failed during {operation}: {message}")]
    Io { operation: String, message: String },
}

impl Error {
    pub fn invalid_input(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidInput {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn unsupported(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Unsupported {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structured_error_round_trips_without_debug_text() {
        let error = Error::BudgetExceeded {
            dimension: BudgetDimension::ResultItems,
            limit: 10,
            consumed: 9,
            requested: 2,
        };
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"kind\":\"budget_exceeded\""));
        assert!(!json.contains("BudgetExceeded"));
        assert_eq!(serde_json::from_str::<Error>(&json).unwrap(), error);
    }
}

use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopError {
    message: String,
}

impl LoopError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for LoopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for LoopError {}

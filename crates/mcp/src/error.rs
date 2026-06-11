use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpError {
    message: String,
}

impl McpError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for McpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for McpError {}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Bridge connection error: {0}")]
    Bridge(String),

    #[error("Engine process error: {0}")]
    Process(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, McpError>;

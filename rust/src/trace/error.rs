//! Error types for the REVM tracer

use std::fmt;
use revm::context::tx::TxEnvBuildError;
use revm::primitives::ruint::FromUintError;
use op_revm::transaction::abstraction::OpBuildError;

/// Main error type for tracing operations
#[derive(Debug)]
pub enum TraceError {
    /// Error building transaction environment
    TxEnvBuild(TxEnvBuildError),
    /// Error building Optimism transaction
    OpTxBuild(OpBuildError),
    /// Error executing transaction
    Execution(String),
    /// Error converting block details
    BlockConversion(FromUintError<u64>),
    /// Error parsing address
    InvalidAddress(String),
    /// Error parsing hex data
    InvalidHexData(String),
    /// Error parsing JSON
    JsonParse(serde_json::Error),
    /// No trace result available
    NoTraceResult,
}

impl fmt::Display for TraceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraceError::TxEnvBuild(e) => write!(f, "Failed to build transaction environment: {:?}", e),
            TraceError::OpTxBuild(e) => write!(f, "Failed to build Optimism transaction: {:?}", e),
            TraceError::Execution(msg) => write!(f, "Transaction execution failed: {}", msg),
            TraceError::BlockConversion(e) => write!(f, "Failed to convert block details: {}", e),
            TraceError::InvalidAddress(addr) => write!(f, "Invalid address: {}", addr),
            TraceError::InvalidHexData(data) => write!(f, "Invalid hex data: {}", data),
            TraceError::JsonParse(e) => write!(f, "Failed to parse JSON: {}", e),
            TraceError::NoTraceResult => write!(f, "No trace result available from inspector"),
        }
    }
}

impl std::error::Error for TraceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TraceError::BlockConversion(e) => Some(e),
            TraceError::JsonParse(e) => Some(e),
            _ => None,
        }
    }
}

// Conversion implementations for ergonomic error handling

impl From<TxEnvBuildError> for TraceError {
    fn from(error: TxEnvBuildError) -> Self {
        TraceError::TxEnvBuild(error)
    }
}

impl From<OpBuildError> for TraceError {
    fn from(error: OpBuildError) -> Self {
        TraceError::OpTxBuild(error)
    }
}

impl From<FromUintError<u64>> for TraceError {
    fn from(error: FromUintError<u64>) -> Self {
        TraceError::BlockConversion(error)
    }
}

impl From<serde_json::Error> for TraceError {
    fn from(error: serde_json::Error) -> Self {
        TraceError::JsonParse(error)
    }
}

impl From<hex::FromHexError> for TraceError {
    fn from(error: hex::FromHexError) -> Self {
        TraceError::InvalidHexData(error.to_string())
    }
}

// Allow conversion to String for backwards compatibility if needed
impl From<TraceError> for String {
    fn from(error: TraceError) -> Self {
        error.to_string()
    }
}

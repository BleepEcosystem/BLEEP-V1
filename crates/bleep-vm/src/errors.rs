

use std::fmt;

#[derive(Debug)]
pub enum VMError {
    ExecutionError(String),
    OptimizationError(String),
    VerificationError(String),
    StateError(String),
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VMError::ExecutionError(e) => write!(f, "Execution error: {}", e),
            VMError::OptimizationError(e) => write!(f, "Optimization error: {}", e),
            VMError::VerificationError(e) => write!(f, "Verification error: {}", e),
            VMError::StateError(e) => write!(f, "State error: {}", e),
        }
    }
}

impl std::error::Error for VMError {}

#[derive(Debug)]
pub enum ExecutionError {
    InstantiationError(String),
    OptimizationError(String),
    CompileError(String),
    ExportError(String),
    RuntimeError(String),
    MemoryError(String),
    Other(String),
}


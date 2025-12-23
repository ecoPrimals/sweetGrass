//! SweetGrass error types.

use thiserror::Error;

/// Errors specific to SweetGrass.
#[derive(Debug, Error)]
pub enum SweetGrassError {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),
    
    // TODO: Add SweetGrass-specific errors
    
    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

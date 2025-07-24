//! Error types for LazyTower

use std::fmt;

/// Errors that can occur when using LazyTower
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LazyTowerError {
    /// Invalid width was provided
    InvalidWidth { width: usize },
    /// Invalid index for proof generation
    InvalidIndex { index: usize, max: usize },
    /// Proof generation not implemented
    ProofGenerationNotImplemented,
}

impl fmt::Display for LazyTowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LazyTowerError::InvalidWidth { width } => {
                write!(f, "Invalid tower width: {}. Width must be greater than 1", width)
            }
            LazyTowerError::InvalidIndex { index, max } => {
                write!(f, "Invalid index {} for proof generation. Valid range: 0..{}", index, max)
            }
            LazyTowerError::ProofGenerationNotImplemented => {
                write!(f, "Proof generation is not yet implemented")
            }
        }
    }
}

impl std::error::Error for LazyTowerError {}
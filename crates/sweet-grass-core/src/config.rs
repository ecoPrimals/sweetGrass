//! SweetGrass configuration.

use serde::{Deserialize, Serialize};
use sourdough_core::config::CommonConfig;

/// Configuration for SweetGrass.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SweetGrassConfig {
    /// Common configuration.
    #[serde(flatten)]
    pub common: CommonConfig,
    
    // TODO: Add SweetGrass-specific configuration
}

impl Default for SweetGrassConfig {
    fn default() -> Self {
        Self {
            common: CommonConfig {
                name: "SweetGrass".to_string(),
                ..CommonConfig::default()
            },
        }
    }
}

//! Machine Learning enhancements for token-optimizer
//! 
//! This module provides ML-powered analysis capabilities including:
//! - Smart context detection
//! - Intelligent impact analysis  
//! - Semantic code search
//! - Pattern detection
//! - Code quality analysis

pub mod config;
pub mod models;
pub mod plugins;
pub mod services;
pub mod external_timeout;
pub mod prompts;
pub mod cache;
pub mod layered_analysis;
pub mod vector_db;
#[cfg(test)]
pub mod real_integration_test;

pub use config::MLConfig;
pub use plugins::*;
pub use services::*;
pub use external_timeout::ExternalTimeoutWrapper;
pub use prompts::StructuredPrompts;
pub use cache::MLResponseCache;

use anyhow::Result;
use uuid::Uuid;

/// Main ML coordinator that manages all ML services
pub struct MLCoordinator {
    config: MLConfig,
    plugin_manager: PluginManager,
    session_id: Uuid,
}

impl MLCoordinator {
    pub fn new(config: MLConfig) -> Self {
        Self {
            config,
            plugin_manager: PluginManager::new(),
            session_id: Uuid::new_v4(),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing ML coordinator with session: {}", self.session_id);
        self.plugin_manager.initialize(&self.config).await?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down ML coordinator");
        self.plugin_manager.shutdown().await?;
        Ok(())
    }

    pub fn get_session_id(&self) -> Uuid {
        self.session_id
    }

    pub fn get_config(&self) -> &MLConfig {
        &self.config
    }

    pub fn get_plugin_manager(&self) -> &PluginManager {
        &self.plugin_manager
    }

    pub fn get_plugin_manager_mut(&mut self) -> &mut PluginManager {
        &mut self.plugin_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ml_coordinator_initialization() {
        let config = MLConfig::for_testing();
        let mut coordinator = MLCoordinator::new(config);
        
        assert!(coordinator.initialize().await.is_ok());
        assert!(coordinator.shutdown().await.is_ok());
    }

    #[test]
    fn test_ml_coordinator_session_id() {
        let config = MLConfig::for_testing();
        let coordinator = MLCoordinator::new(config);
        
        let session_id = coordinator.get_session_id();
        assert!(!session_id.is_nil());
    }
}
//! ML services for enhanced code analysis

use anyhow::Result;
use std::sync::Arc;

use crate::ml::config::MLConfig;
use crate::ml::plugins::PluginManager;

pub mod context;
pub mod impact_analysis;
pub mod pattern;
pub mod search;
pub mod optimization;
pub mod enhanced_search;

pub use context::SmartContextService;
pub use impact_analysis::ImpactAnalysisService;
pub use pattern::PatternDetectionService;
pub use search::SemanticSearchService;
pub use optimization::TokenOptimizationService;
pub use enhanced_search::{EnhancedSearchService, SearchRequest, SearchType, SearchFilters, SearchOptions, SearchResponse, CodeIndexEntry};

/// Main ML service coordinator
pub struct MLService {
    config: MLConfig,
    plugin_manager: Arc<PluginManager>,
    context_service: SmartContextService,
    impact_service: ImpactAnalysisService,
    pattern_service: PatternDetectionService,
    search_service: SemanticSearchService,
    optimization_service: TokenOptimizationService,
}

impl MLService {
    pub fn new(config: MLConfig, plugin_manager: Arc<PluginManager>) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            plugin_manager: plugin_manager.clone(),
            context_service: SmartContextService::new(config.clone(), plugin_manager.clone())?,
            impact_service: ImpactAnalysisService::new(config.clone(), plugin_manager.clone()),
            pattern_service: PatternDetectionService::new(config.clone(), plugin_manager.clone()),
            search_service: SemanticSearchService::new(config.clone(), plugin_manager.clone()),
            optimization_service: TokenOptimizationService::new(config.clone(), plugin_manager.clone()),
        })
    }

    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing ML service");
        
        // Initialize all sub-services
        self.context_service.initialize().await?;
        self.impact_service.initialize().await?;
        self.pattern_service.initialize().await?;
        self.search_service.initialize().await?;
        self.optimization_service.initialize().await?;
        
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down ML service");
        
        // Shutdown all sub-services
        self.context_service.shutdown().await?;
        self.impact_service.shutdown().await?;
        self.pattern_service.shutdown().await?;
        self.search_service.shutdown().await?;
        self.optimization_service.shutdown().await?;
        
        Ok(())
    }

    // Service getters
    pub fn context_service(&self) -> &SmartContextService {
        &self.context_service
    }

    pub fn impact_service(&self) -> &ImpactAnalysisService {
        &self.impact_service
    }

    pub fn pattern_service(&self) -> &PatternDetectionService {
        &self.pattern_service
    }

    pub fn search_service(&self) -> &SemanticSearchService {
        &self.search_service
    }

    pub fn optimization_service(&self) -> &TokenOptimizationService {
        &self.optimization_service
    }

    pub fn config(&self) -> &MLConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::config::MLConfig;

    #[tokio::test]
    async fn test_ml_service_creation() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = MLService::new(config, plugin_manager).unwrap();
        
        // Test that services can be created successfully
        // Services start uninitialized and must be initialized before use
        assert!(!service.context_service.is_ready());
        assert!(!service.impact_service.is_ready());
        assert!(!service.pattern_service.is_ready());
        assert!(!service.search_service.is_ready());
        assert!(!service.optimization_service.is_ready());
    }

    #[tokio::test]
    async fn test_ml_service_initialization() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let mut service = MLService::new(config, plugin_manager).unwrap();
        
        // ML service should initialize successfully with graceful fallback
        // when plugins are not available
        assert!(service.initialize().await.is_ok());
        
        // Verify all services are ready after initialization
        assert!(service.context_service.is_ready());
        assert!(service.impact_service.is_ready());
        assert!(service.pattern_service.is_ready());
        assert!(service.search_service.is_ready());
        assert!(service.optimization_service.is_ready());
    }
}

#[cfg(test)]
mod e2e_test;

#[cfg(test)]
mod real_project_test;

#[cfg(test)]
mod real_ml_test;

#[cfg(test)]
mod pattern_e2e_test;

#[cfg(test)]
mod memory_leak_test;

#[cfg(test)]
mod real_vram_test;

#[cfg(test)]
mod semantic_scoring_test;
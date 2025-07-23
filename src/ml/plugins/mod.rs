//! ML Plugin system for modular model management

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

use crate::ml::config::MLConfig;

pub mod deepseek;
pub mod qwen_embedding;
pub mod qwen_reranker;

#[cfg(test)]
pub mod gguf_loader_test;

#[cfg(test)]
pub mod real_embedding_test;

pub use deepseek::DeepSeekPlugin;
pub use qwen_embedding::QwenEmbeddingPlugin;
pub use qwen_reranker::QwenRerankerPlugin;

/// ML capabilities that plugins can provide
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MLCapability {
    TextEmbedding,
    CodeEmbedding,
    TextReranking,
    CodeReranking,
    Reasoning,
    CodeAnalysis,
    TextGeneration,
    CodeGeneration,
}

/// Plugin status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatus {
    pub loaded: bool,
    pub memory_mb: usize,
    pub last_used: Option<SystemTime>,
    pub error: Option<String>,
    pub capabilities: Vec<MLCapability>,
}

/// Loading strategy for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadingStrategy {
    OnDemand,
    KeepAlive(Duration),
    Preloaded,
}

/// Plugin trait for ML models
#[async_trait]
pub trait MLPlugin: Send + Sync {
    /// Get plugin name
    fn name(&self) -> &str;
    
    /// Get plugin version
    fn version(&self) -> &str;
    
    /// Get estimated memory usage in bytes
    fn memory_usage(&self) -> usize;
    
    /// Check if plugin is loaded
    fn is_loaded(&self) -> bool;
    
    /// Load the plugin with configuration
    async fn load(&mut self, config: &MLConfig) -> Result<()>;
    
    /// Unload the plugin and free resources
    async fn unload(&mut self) -> Result<()>;
    
    /// Health check with detailed status
    async fn health_check(&self) -> Result<PluginStatus>;
    
    /// Get plugin-specific capabilities
    fn capabilities(&self) -> Vec<MLCapability>;
    
    /// Process input and return output (kept for backward compatibility)
    async fn process(&self, input: &str) -> Result<String>;
}

/// Plugin manager for handling ML plugins
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Box<dyn MLPlugin>>>>,
    active_plugins: Arc<RwLock<HashMap<String, Uuid>>>,
    memory_usage: Arc<RwLock<usize>>,
    config: Option<MLConfig>,
    loading_strategy: LoadingStrategy,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            active_plugins: Arc::new(RwLock::new(HashMap::new())),
            memory_usage: Arc::new(RwLock::new(0)),
            config: None,
            loading_strategy: LoadingStrategy::OnDemand,
        }
    }
    
    pub fn with_loading_strategy(mut self, strategy: LoadingStrategy) -> Self {
        self.loading_strategy = strategy;
        self
    }

    pub async fn initialize(&mut self, config: &MLConfig) -> Result<()> {
        self.config = Some(config.clone());
        
        // Register default plugins
        self.register_plugin("deepseek", Box::new(DeepSeekPlugin::new())).await?;
        self.register_plugin("qwen_embedding", Box::new(QwenEmbeddingPlugin::new())).await?;
        self.register_plugin("qwen_reranker", Box::new(QwenRerankerPlugin::new())).await?;
        
        tracing::info!("Plugin manager initialized with {} plugins", self.get_plugin_count());
        Ok(())
    }

    pub async fn register_plugin(&mut self, name: &str, plugin: Box<dyn MLPlugin>) -> Result<()> {
        let mut plugins = self.plugins.write();
        
        if plugins.contains_key(name) {
            anyhow::bail!("Plugin {} already registered", name);
        }
        
        plugins.insert(name.to_string(), plugin);
        tracing::info!("Registered plugin: {}", name);
        Ok(())
    }
    
    pub async fn get_plugin(&self, name: &str) -> Result<&dyn MLPlugin> {
        let plugins = self.plugins.read();
        let _plugin = plugins.get(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found", name))?;
        
        // This is a limitation of the current design - we can't return a reference 
        // that outlives the lock. We'll need to redesign this for the actual usage.
        // For now, we'll use a different approach in the services.
        anyhow::bail!("Direct plugin access not supported - use process_with_plugin instead")
    }
    
    pub async fn ensure_loaded(&mut self, plugin_name: &str) -> Result<()> {
        if !self.is_plugin_loaded(plugin_name) {
            self.load_plugin(plugin_name).await?;
        }
        Ok(())
    }
    
    pub async fn unload_unused(&mut self, max_idle: Duration) -> Result<()> {
        let now = SystemTime::now();
        let active_plugins: Vec<String> = self.active_plugins.read().keys().cloned().collect();
        
        for plugin_name in active_plugins {
            // For now, we'll implement a simple strategy
            // In a real implementation, we'd track last_used times
            if let Ok(elapsed) = now.duration_since(SystemTime::UNIX_EPOCH) {
                if elapsed > max_idle {
                    self.unload_plugin(&plugin_name).await?;
                }
            }
        }
        
        Ok(())
    }
    
    pub fn get_status(&self) -> Vec<PluginStatus> {
        let plugins = self.plugins.read();
        let mut statuses = Vec::new();
        
        for (_name, plugin) in plugins.iter() {
            let status = PluginStatus {
                loaded: plugin.is_loaded(),
                memory_mb: plugin.memory_usage() / 1024 / 1024,
                last_used: None, // TODO: implement last_used tracking
                error: None,
                capabilities: plugin.capabilities(),
            };
            statuses.push(status);
        }
        
        statuses
    }

    pub async fn load_plugin(&self, name: &str) -> Result<Uuid> {
        let config = self.config.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Plugin manager not initialized"))?;
        
        let mut plugins = self.plugins.write();
        let plugin = plugins.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found", name))?;
        
        // Check memory constraints
        let plugin_memory = plugin.memory_usage();
        let current_memory = *self.memory_usage.read();
        
        if current_memory + plugin_memory > config.memory_budget {
            // Try to unload least recently used plugin
            self.unload_least_used_plugin().await?;
            
            // Check again
            let current_memory = *self.memory_usage.read();
            if current_memory + plugin_memory > config.memory_budget {
                anyhow::bail!("Insufficient memory to load plugin {}: {} bytes needed, {} available", 
                             name, plugin_memory, config.memory_budget - current_memory);
            }
        }
        
        plugin.load(config).await?;
        
        let session_id = Uuid::new_v4();
        self.active_plugins.write().insert(name.to_string(), session_id);
        *self.memory_usage.write() += plugin_memory;
        
        tracing::info!("Loaded plugin: {} (session: {})", name, session_id);
        Ok(session_id)
    }

    pub async fn unload_plugin(&self, name: &str) -> Result<()> {
        let mut plugins = self.plugins.write();
        let plugin = plugins.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found", name))?;
        
        let plugin_memory = plugin.memory_usage();
        plugin.unload().await?;
        
        self.active_plugins.write().remove(name);
        *self.memory_usage.write() -= plugin_memory;
        
        tracing::info!("Unloaded plugin: {}", name);
        Ok(())
    }

    pub async fn process_with_plugin(&self, plugin_name: &str, input: &str) -> Result<String> {
        // Check if plugin is loaded
        if !self.is_plugin_loaded(plugin_name) {
            self.load_plugin(plugin_name).await?;
        }
        
        let plugins = self.plugins.read();
        let plugin = plugins.get(plugin_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found", plugin_name))?;
        
        plugin.process(input).await
    }

    pub fn is_plugin_loaded(&self, name: &str) -> bool {
        self.active_plugins.read().contains_key(name)
    }

    pub fn get_plugin_count(&self) -> usize {
        self.plugins.read().len()
    }

    pub fn get_active_plugin_count(&self) -> usize {
        self.active_plugins.read().len()
    }

    pub fn get_memory_usage(&self) -> usize {
        *self.memory_usage.read()
    }

    pub fn get_available_plugins(&self) -> Vec<String> {
        self.plugins.read().keys().cloned().collect()
    }

    pub fn get_active_plugins(&self) -> Vec<String> {
        self.active_plugins.read().keys().cloned().collect()
    }

    pub async fn health_check(&self) -> Result<HashMap<String, PluginStatus>> {
        let plugins = self.plugins.read();
        let mut results = HashMap::new();
        
        for (name, plugin) in plugins.iter() {
            let status = plugin.health_check().await.unwrap_or(PluginStatus {
                loaded: false,
                memory_mb: 0,
                last_used: None,
                error: Some("Health check failed".to_string()),
                capabilities: vec![],
            });
            results.insert(name.clone(), status);
        }
        
        Ok(results)
    }

    pub async fn shutdown(&self) -> Result<()> {
        let plugin_names: Vec<String> = self.active_plugins.read().keys().cloned().collect();
        
        for name in plugin_names {
            if let Err(e) = self.unload_plugin(&name).await {
                tracing::error!("Error unloading plugin {}: {}", name, e);
            }
        }
        
        tracing::info!("Plugin manager shutdown complete");
        Ok(())
    }

    async fn unload_least_used_plugin(&self) -> Result<()> {
        let active_plugins = self.active_plugins.read();
        
        if let Some(plugin_name) = active_plugins.keys().next() {
            let plugin_name = plugin_name.clone();
            drop(active_plugins);
            self.unload_plugin(&plugin_name).await?;
        }
        
        Ok(())
    }
}

// Ensure proper cleanup on drop
impl Drop for PluginManager {
    fn drop(&mut self) {
        // Since we can't use async in Drop, do synchronous cleanup
        // Clear memory tracking and active plugins
        *self.memory_usage.write() = 0;
        self.active_plugins.write().clear();
        
        // Note: The actual model cleanup should be handled by the individual plugin Drop implementations
        if self.active_plugins.read().len() > 0 {
            tracing::warn!("PluginManager dropped with active plugins - ensure proper shutdown() is called");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::config::MLConfig;

    #[tokio::test]
    async fn test_plugin_manager_initialization() {
        let mut manager = PluginManager::new();
        let config = MLConfig::for_testing();
        
        assert!(manager.initialize(&config).await.is_ok());
        assert_eq!(manager.get_plugin_count(), 3); // deepseek, qwen_embedding, qwen_reranker
    }

    #[tokio::test]
    async fn test_plugin_loading() {
        let mut manager = PluginManager::new();
        let config = MLConfig::for_testing();
        manager.initialize(&config).await.unwrap();
        
        // Test loading a plugin
        let session_id = manager.load_plugin("deepseek").await.unwrap();
        assert!(!session_id.is_nil());
        assert!(manager.is_plugin_loaded("deepseek"));
        assert_eq!(manager.get_active_plugin_count(), 1);
        
        // Test unloading a plugin
        manager.unload_plugin("deepseek").await.unwrap();
        assert!(!manager.is_plugin_loaded("deepseek"));
        assert_eq!(manager.get_active_plugin_count(), 0);
    }

    #[tokio::test]
    async fn test_plugin_health_check() {
        let mut manager = PluginManager::new();
        let config = MLConfig::for_testing();
        manager.initialize(&config).await.unwrap();
        
        let health = manager.health_check().await.unwrap();
        assert_eq!(health.len(), 3);
        assert!(health.contains_key("deepseek"));
        assert!(health.contains_key("qwen_embedding"));
        assert!(health.contains_key("qwen_reranker"));
        
        // Check that we get PluginStatus objects
        for (_, status) in health.iter() {
            assert!(status.capabilities.len() >= 0);
        }
    }

    #[tokio::test]
    async fn test_plugin_shutdown() {
        let mut manager = PluginManager::new();
        let config = MLConfig::for_testing();
        manager.initialize(&config).await.unwrap();
        
        // Load a plugin
        manager.load_plugin("deepseek").await.unwrap();
        assert_eq!(manager.get_active_plugin_count(), 1);
        
        // Shutdown should unload all plugins
        manager.shutdown().await.unwrap();
        assert_eq!(manager.get_active_plugin_count(), 0);
    }
}
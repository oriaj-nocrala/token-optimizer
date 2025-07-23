//! ML configuration and resource management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Memory budget in bytes
pub const DEFAULT_MEMORY_BUDGET: usize = 6_000_000_000; // 6GB
pub const MIN_MEMORY_BUDGET: usize = 2_000_000_000; // 2GB
pub const MAX_MEMORY_BUDGET: usize = 12_000_000_000; // 12GB

/// Model loading strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelLoadingStrategy {
    /// Load models on-demand when needed
    OnDemand,
    /// Preload all models at startup
    Preload,
    /// Keep models in memory after first use
    Persistent,
}

/// Quantization levels for memory optimization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum QuantizationLevel {
    /// 4-bit quantization (lowest memory, fastest)
    Q4_K_M,
    /// 6-bit quantization (balanced)
    Q6_K,
    /// 8-bit quantization (higher quality)
    Q8_0,
    /// 16-bit quantization (highest quality, most memory)
    F16,
}

/// ML configuration for resource management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfig {
    /// Memory budget in bytes
    pub memory_budget: usize,
    /// Model loading strategy
    pub model_loading: ModelLoadingStrategy,
    /// Quantization level
    pub quantization: QuantizationLevel,
    /// Maximum concurrent models
    pub max_concurrent_models: usize,
    /// Model cache directory
    pub model_cache_dir: PathBuf,
    /// Enable GPU acceleration if available
    pub use_gpu: bool,
    /// GPU memory fraction to use (0.0 to 1.0)
    pub gpu_memory_fraction: f32,
    /// Timeout for model operations in seconds
    pub operation_timeout: u64,
    /// User configurable timeout range (min, max) in seconds
    pub user_timeout_range: (u64, u64),
    /// External process timeout for llama-cli calls in seconds
    pub external_process_timeout: u64,
    /// Reasoning-specific timeout for DeepSeek thinking in seconds
    pub reasoning_timeout: u64,
    /// Embedding-specific timeout for Qwen operations in seconds
    pub embedding_timeout: u64,
    /// Enable external timeout command wrapper
    pub enable_external_timeout: bool,
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            memory_budget: DEFAULT_MEMORY_BUDGET,
            model_loading: ModelLoadingStrategy::OnDemand,
            quantization: QuantizationLevel::Q6_K,
            max_concurrent_models: 1,
            model_cache_dir: PathBuf::from(".cache/ml-models"),
            use_gpu: true,
            gpu_memory_fraction: 0.8,
            operation_timeout: 30,
            user_timeout_range: (120, 300), // 2-5 minutes as recommended
            external_process_timeout: 300,  // 5 minutes for external llama-cli
            reasoning_timeout: 240,         // 4 minutes for DeepSeek thinking
            embedding_timeout: 60,          // 1 minute for Qwen embeddings
            enable_external_timeout: true,  // Enable external timeout control
        }
    }
}

impl MLConfig {
    /// Create config optimized for 8GB VRAM
    pub fn for_8gb_vram() -> Self {
        Self {
            memory_budget: 6_000_000_000,
            model_loading: ModelLoadingStrategy::OnDemand,
            quantization: QuantizationLevel::Q6_K,
            max_concurrent_models: 1,
            model_cache_dir: PathBuf::from(".cache/ml-models"),
            use_gpu: true,
            gpu_memory_fraction: 0.75,
            operation_timeout: 30,
            user_timeout_range: (120, 300),
            external_process_timeout: 300,
            reasoning_timeout: 240,
            embedding_timeout: 60,
            enable_external_timeout: true,
        }
    }

    /// Create config optimized for 16GB VRAM
    pub fn for_16gb_vram() -> Self {
        Self {
            memory_budget: 12_000_000_000,
            model_loading: ModelLoadingStrategy::Persistent,
            quantization: QuantizationLevel::Q8_0,
            max_concurrent_models: 2,
            model_cache_dir: PathBuf::from(".cache/ml-models"),
            use_gpu: true,
            gpu_memory_fraction: 0.8,
            operation_timeout: 30,
            user_timeout_range: (180, 360), // Higher timeouts for 16GB systems
            external_process_timeout: 360,
            reasoning_timeout: 300,
            embedding_timeout: 90,
            enable_external_timeout: true,
        }
    }

    /// Create config for CPU-only systems
    pub fn for_cpu_only() -> Self {
        Self {
            memory_budget: 8_000_000_000,
            model_loading: ModelLoadingStrategy::OnDemand,
            quantization: QuantizationLevel::Q4_K_M,
            max_concurrent_models: 1,
            model_cache_dir: PathBuf::from(".cache/ml-models"),
            use_gpu: false,
            gpu_memory_fraction: 0.0,
            operation_timeout: 60,
            user_timeout_range: (180, 600), // Longer timeouts for CPU
            external_process_timeout: 600,  // 10 minutes for CPU inference
            reasoning_timeout: 480,         // 8 minutes for CPU DeepSeek
            embedding_timeout: 120,         // 2 minutes for CPU embeddings
            enable_external_timeout: true,
        }
    }

    /// Create minimal config for testing
    pub fn for_testing() -> Self {
        Self {
            memory_budget: 10_000_000_000, // 10GB for testing
            model_loading: ModelLoadingStrategy::OnDemand,
            quantization: QuantizationLevel::Q4_K_M,
            max_concurrent_models: 1,
            model_cache_dir: PathBuf::from(".cache/test-models"),
            use_gpu: false,
            gpu_memory_fraction: 0.0,
            operation_timeout: 10,
            user_timeout_range: (30, 60),   // Short timeouts for testing
            external_process_timeout: 60,
            reasoning_timeout: 45,
            embedding_timeout: 30,
            enable_external_timeout: false, // Disable external timeout in tests
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.memory_budget < MIN_MEMORY_BUDGET {
            anyhow::bail!("Memory budget too low: {} bytes (minimum: {})", 
                         self.memory_budget, MIN_MEMORY_BUDGET);
        }

        if self.memory_budget > MAX_MEMORY_BUDGET {
            anyhow::bail!("Memory budget too high: {} bytes (maximum: {})", 
                         self.memory_budget, MAX_MEMORY_BUDGET);
        }

        if self.max_concurrent_models == 0 {
            anyhow::bail!("Max concurrent models must be at least 1");
        }

        if self.use_gpu && (self.gpu_memory_fraction <= 0.0 || self.gpu_memory_fraction > 1.0) {
            anyhow::bail!("GPU memory fraction must be between 0.0 and 1.0");
        }

        if self.operation_timeout == 0 {
            anyhow::bail!("Operation timeout must be greater than 0");
        }

        // Validate timeout range
        if self.user_timeout_range.0 >= self.user_timeout_range.1 {
            anyhow::bail!("User timeout range invalid: min {} >= max {}", 
                         self.user_timeout_range.0, self.user_timeout_range.1);
        }

        if self.user_timeout_range.0 < 30 {
            anyhow::bail!("User timeout range minimum too low: {} seconds (minimum: 30)", 
                         self.user_timeout_range.0);
        }

        if self.external_process_timeout == 0 {
            anyhow::bail!("External process timeout must be greater than 0");
        }

        if self.reasoning_timeout == 0 {
            anyhow::bail!("Reasoning timeout must be greater than 0");
        }

        if self.embedding_timeout == 0 {
            anyhow::bail!("Embedding timeout must be greater than 0");
        }

        Ok(())
    }

    /// Get estimated memory usage for a model
    pub fn estimate_model_memory(&self, model_size_gb: f64) -> usize {
        let multiplier = match self.quantization {
            QuantizationLevel::Q4_K_M => 0.25,
            QuantizationLevel::Q6_K => 0.375,
            QuantizationLevel::Q8_0 => 0.5,
            QuantizationLevel::F16 => 1.0,
        };

        (model_size_gb * multiplier * 1_000_000_000.0) as usize
    }

    /// Check if a model can fit in memory budget
    pub fn can_fit_model(&self, model_size_gb: f64) -> bool {
        let estimated_memory = self.estimate_model_memory(model_size_gb);
        estimated_memory <= self.memory_budget
    }

    /// Get quantization suffix for model filenames
    pub fn get_quantization_suffix(&self) -> &'static str {
        match self.quantization {
            QuantizationLevel::Q4_K_M => "Q4_K_M",
            QuantizationLevel::Q6_K => "Q6_K",
            QuantizationLevel::Q8_0 => "Q8_0",
            QuantizationLevel::F16 => "F16",
        }
    }

    /// Get timeout for DeepSeek reasoning operations
    pub fn get_reasoning_timeout(&self) -> u64 {
        self.reasoning_timeout
    }

    /// Get timeout for Qwen embedding operations
    pub fn get_embedding_timeout(&self) -> u64 {
        self.embedding_timeout
    }

    /// Get timeout for external process calls
    pub fn get_external_process_timeout(&self) -> u64 {
        self.external_process_timeout
    }

    /// Check if user timeout is within allowed range
    pub fn is_valid_user_timeout(&self, timeout: u64) -> bool {
        timeout >= self.user_timeout_range.0 && timeout <= self.user_timeout_range.1
    }

    /// Get user timeout clamped to valid range
    pub fn clamp_user_timeout(&self, timeout: u64) -> u64 {
        timeout.max(self.user_timeout_range.0).min(self.user_timeout_range.1)
    }

    /// Check if external timeout command wrapper is enabled
    pub fn is_external_timeout_enabled(&self) -> bool {
        self.enable_external_timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MLConfig::default();
        assert_eq!(config.memory_budget, DEFAULT_MEMORY_BUDGET);
        assert_eq!(config.model_loading, ModelLoadingStrategy::OnDemand);
        assert_eq!(config.quantization, QuantizationLevel::Q6_K);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_8gb_vram_config() {
        let config = MLConfig::for_8gb_vram();
        assert_eq!(config.memory_budget, 6_000_000_000);
        assert_eq!(config.max_concurrent_models, 1);
        assert_eq!(config.gpu_memory_fraction, 0.75);
        assert_eq!(config.reasoning_timeout, 240);
        assert_eq!(config.embedding_timeout, 60);
        assert!(config.enable_external_timeout);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cpu_only_config() {
        let config = MLConfig::for_cpu_only();
        assert!(!config.use_gpu);
        assert_eq!(config.gpu_memory_fraction, 0.0);
        assert_eq!(config.quantization, QuantizationLevel::Q4_K_M);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = MLConfig::default();
        
        // Test memory budget validation
        config.memory_budget = MIN_MEMORY_BUDGET - 1;
        assert!(config.validate().is_err());
        
        config.memory_budget = MAX_MEMORY_BUDGET + 1;
        assert!(config.validate().is_err());
        
        // Test max concurrent models validation
        config.memory_budget = DEFAULT_MEMORY_BUDGET;
        config.max_concurrent_models = 0;
        assert!(config.validate().is_err());
        
        // Test GPU memory fraction validation
        config.max_concurrent_models = 1;
        config.use_gpu = true;
        config.gpu_memory_fraction = 0.0;
        assert!(config.validate().is_err());
        
        config.gpu_memory_fraction = 1.1;
        assert!(config.validate().is_err());
        
        // Test timeout range validation
        config.gpu_memory_fraction = 0.8;
        config.user_timeout_range = (300, 120); // Invalid: min > max
        assert!(config.validate().is_err());
        
        config.user_timeout_range = (15, 300); // Invalid: min too low
        assert!(config.validate().is_err());
        
        // Test individual timeout validation
        config.user_timeout_range = (120, 300);
        config.external_process_timeout = 0;
        assert!(config.validate().is_err());
        
        config.external_process_timeout = 300;
        config.reasoning_timeout = 0;
        assert!(config.validate().is_err());
        
        config.reasoning_timeout = 240;
        config.embedding_timeout = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_memory_estimation() {
        let config = MLConfig::default();
        
        // Test Q6_K quantization (37.5% of original)
        let estimated = config.estimate_model_memory(8.0); // 8GB model
        assert_eq!(estimated, 3_000_000_000); // 3GB
        
        // Test Q4_K_M quantization (25% of original)
        let mut config_q4 = config.clone();
        config_q4.quantization = QuantizationLevel::Q4_K_M;
        let estimated_q4 = config_q4.estimate_model_memory(8.0);
        assert_eq!(estimated_q4, 2_000_000_000); // 2GB
    }

    #[test]
    fn test_model_fit_check() {
        let config = MLConfig::for_8gb_vram();
        
        // 8GB model with Q6_K should fit (3GB estimated)
        assert!(config.can_fit_model(8.0));
        
        // 20GB model with Q6_K would be 7.5GB estimated, exceeds 6GB budget
        assert!(!config.can_fit_model(20.0));
    }

    #[test]
    fn test_quantization_suffix() {
        let config = MLConfig::default();
        assert_eq!(config.get_quantization_suffix(), "Q6_K");
        
        let mut config_q4 = config;
        config_q4.quantization = QuantizationLevel::Q4_K_M;
        assert_eq!(config_q4.get_quantization_suffix(), "Q4_K_M");
    }

    #[test]
    fn test_timeout_helpers() {
        let config = MLConfig::default();
        
        // Test timeout getters
        assert_eq!(config.get_reasoning_timeout(), 240);
        assert_eq!(config.get_embedding_timeout(), 60);
        assert_eq!(config.get_external_process_timeout(), 300);
        
        // Test user timeout validation
        assert!(config.is_valid_user_timeout(150)); // Within range
        assert!(!config.is_valid_user_timeout(100)); // Below min
        assert!(!config.is_valid_user_timeout(400)); // Above max
        
        // Test user timeout clamping
        assert_eq!(config.clamp_user_timeout(100), 120); // Clamped to min
        assert_eq!(config.clamp_user_timeout(200), 200); // Within range
        assert_eq!(config.clamp_user_timeout(400), 300); // Clamped to max
        
        // Test external timeout check
        assert!(config.is_external_timeout_enabled());
        
        let test_config = MLConfig::for_testing();
        assert!(!test_config.is_external_timeout_enabled());
    }
}
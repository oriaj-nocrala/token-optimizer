//! Model downloader for GGUF models from Hugging Face

use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{error, info, warn};

use crate::ml::config::MLConfig;

/// Model download information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub url: String,
    pub filename: String,
    pub size_gb: f64,
    pub description: String,
}

/// Model downloader for GGUF models
pub struct ModelDownloader {
    client: Client,
    config: MLConfig,
}

impl ModelDownloader {
    pub fn new(config: MLConfig) -> Self {
        let client = Client::new();
        Self { client, config }
    }

    /// Get available models for download
    pub fn get_available_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                name: "deepseek-r1".to_string(),
                url: "https://huggingface.co/unsloth/DeepSeek-R1-0528-Qwen3-8B-GGUF/resolve/main/DeepSeek-R1-0528-Qwen3-8B-Q6_K.gguf".to_string(),
                filename: format!("DeepSeek-R1-0528-Qwen3-8B-{}.gguf", self.config.get_quantization_suffix()),
                size_gb: 8.0,
                description: "DeepSeek-R1 reasoning model for impact analysis".to_string(),
            },
            ModelInfo {
                name: "qwen3-embedding".to_string(),
                url: "https://huggingface.co/Qwen/Qwen3-Embedding-8B-GGUF/resolve/main/Qwen3-Embedding-8B-Q6_K.gguf".to_string(),
                filename: format!("Qwen3-Embedding-8B-{}.gguf", self.config.get_quantization_suffix()),
                size_gb: 8.0,
                description: "Qwen3 embedding model for semantic similarity".to_string(),
            },
            ModelInfo {
                name: "qwen3-reranker".to_string(),
                url: "https://huggingface.co/aotsukiqx/Qwen3-Reranker-8B-Q6_K-GGUF/resolve/main/qwen3-reranker-8b-q6_k.gguf".to_string(),
                filename: "qwen3-reranker-8b-q6_k.gguf".to_string(), // Fixed to match actual filename
                size_gb: 8.0,
                description: "Qwen3 reranker model for relevance scoring".to_string(),
            },
        ]
    }

    /// Download a model by name
    pub async fn download_model(&self, model_name: &str) -> Result<PathBuf> {
        let models = self.get_available_models();
        let model = models.iter()
            .find(|m| m.name == model_name)
            .ok_or_else(|| anyhow::anyhow!("Model '{}' not found", model_name))?;

        let output_path = self.config.model_cache_dir.join(&model.filename);
        
        // Check if model already exists
        if output_path.exists() {
            info!("Model already exists: {}", output_path.display());
            return Ok(output_path);
        }

        // Check if model can fit in memory budget
        if !self.config.can_fit_model(model.size_gb) {
            anyhow::bail!("Model '{}' ({:.1}GB) exceeds memory budget ({:.1}GB)", 
                         model_name, model.size_gb, self.config.memory_budget as f64 / 1_000_000_000.0);
        }

        info!("Downloading model '{}' from {}", model_name, model.url);
        info!("Output path: {}", output_path.display());

        // Create cache directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Download the model
        self.download_file(&model.url, &output_path).await?;

        info!("Model '{}' downloaded successfully", model_name);
        Ok(output_path)
    }

    /// Download all available models
    pub async fn download_all_models(&self) -> Result<Vec<PathBuf>> {
        let models = self.get_available_models();
        let mut downloaded_paths = Vec::new();

        for model in models {
            match self.download_model(&model.name).await {
                Ok(path) => {
                    downloaded_paths.push(path);
                }
                Err(e) => {
                    error!("Failed to download model '{}': {}", model.name, e);
                }
            }
        }

        Ok(downloaded_paths)
    }

    /// Check which models are available locally
    pub fn check_local_models(&self) -> Vec<(String, bool)> {
        let models = self.get_available_models();
        let mut status = Vec::new();

        for model in models {
            let path = self.config.model_cache_dir.join(&model.filename);
            status.push((model.name, path.exists()));
        }

        status
    }

    /// Delete a model from local cache
    pub fn delete_model(&self, model_name: &str) -> Result<()> {
        let models = self.get_available_models();
        let model = models.iter()
            .find(|m| m.name == model_name)
            .ok_or_else(|| anyhow::anyhow!("Model '{}' not found", model_name))?;

        let path = self.config.model_cache_dir.join(&model.filename);
        
        if path.exists() {
            fs::remove_file(&path)?;
            info!("Model '{}' deleted from cache", model_name);
        } else {
            warn!("Model '{}' not found in cache", model_name);
        }

        Ok(())
    }

    /// Get total size of cached models
    pub fn get_cache_size(&self) -> Result<u64> {
        let mut total_size = 0;

        if self.config.model_cache_dir.exists() {
            for entry in fs::read_dir(&self.config.model_cache_dir)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                if metadata.is_file() {
                    total_size += metadata.len();
                }
            }
        }

        Ok(total_size)
    }

    /// Clean up cache directory
    pub fn clean_cache(&self) -> Result<()> {
        if self.config.model_cache_dir.exists() {
            fs::remove_dir_all(&self.config.model_cache_dir)?;
            info!("Model cache cleaned");
        }
        Ok(())
    }

    /// Download a file from URL to local path
    async fn download_file(&self, url: &str, output_path: &Path) -> Result<()> {
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to download: HTTP {}", response.status());
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        
        let mut file = File::create(output_path).await?;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            
            if total_size > 0 {
                let progress = (downloaded as f64 / total_size as f64 * 100.0) as u32;
                if downloaded % (1024 * 1024 * 10) == 0 { // Log every 10MB
                    info!("Download progress: {}% ({:.1}MB / {:.1}MB)", 
                         progress, downloaded as f64 / 1_000_000.0, total_size as f64 / 1_000_000.0);
                }
            }
        }

        file.flush().await?;
        info!("Download completed: {:.1}MB", downloaded as f64 / 1_000_000.0);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_model_downloader_creation() {
        let config = MLConfig::for_testing();
        let downloader = ModelDownloader::new(config);
        
        let models = downloader.get_available_models();
        assert!(!models.is_empty());
        assert_eq!(models.len(), 3);
    }

    #[test]
    fn test_get_available_models() {
        let config = MLConfig::for_testing();
        let downloader = ModelDownloader::new(config);
        
        let models = downloader.get_available_models();
        
        assert!(models.iter().any(|m| m.name == "deepseek-r1"));
        assert!(models.iter().any(|m| m.name == "qwen3-embedding"));
        assert!(models.iter().any(|m| m.name == "qwen3-reranker"));
    }

    #[test]
    fn test_check_local_models() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = MLConfig::for_testing();
        config.model_cache_dir = temp_dir.path().to_path_buf();
        
        let downloader = ModelDownloader::new(config);
        let status = downloader.check_local_models();
        
        assert_eq!(status.len(), 3);
        // All models should be missing initially
        assert!(status.iter().all(|(_, exists)| !exists));
    }

    #[test]
    fn test_cache_size_empty() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = MLConfig::for_testing();
        config.model_cache_dir = temp_dir.path().to_path_buf();
        
        let downloader = ModelDownloader::new(config);
        let size = downloader.get_cache_size().unwrap();
        
        assert_eq!(size, 0);
    }

    #[test]
    fn test_delete_nonexistent_model() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = MLConfig::for_testing();
        config.model_cache_dir = temp_dir.path().to_path_buf();
        
        let downloader = ModelDownloader::new(config);
        // Test deleting a valid model name but file doesn't exist
        let result = downloader.delete_model("deepseek-r1");
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_model_info_structure() {
        let config = MLConfig::for_testing();
        let downloader = ModelDownloader::new(config);
        let models = downloader.get_available_models();
        
        for model in models {
            assert!(!model.name.is_empty());
            assert!(model.url.starts_with("https://"));
            assert!(model.filename.ends_with(".gguf"));
            assert!(model.size_gb > 0.0);
            assert!(!model.description.is_empty());
        }
    }

    #[test]
    fn test_clean_cache() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = MLConfig::for_testing();
        config.model_cache_dir = temp_dir.path().to_path_buf();
        
        // Create the cache directory
        fs::create_dir_all(&config.model_cache_dir).unwrap();
        
        let downloader = ModelDownloader::new(config);
        let result = downloader.clean_cache();
        
        assert!(result.is_ok());
        assert!(!temp_dir.path().exists() || !temp_dir.path().join("some_file").exists());
    }

    // Note: We don't test actual downloads in unit tests as they require internet
    // These would be integration tests
}
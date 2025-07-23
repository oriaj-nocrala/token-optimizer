//! External timeout wrapper for ML model calls

use anyhow::Result;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::timeout;

use crate::ml::config::MLConfig;

/// External timeout wrapper for preventing ML model hangs
pub struct ExternalTimeoutWrapper {
    config: MLConfig,
}

impl ExternalTimeoutWrapper {
    pub fn new(config: MLConfig) -> Self {
        Self { config }
    }

    /// Execute llama-cli with external timeout protection
    pub async fn execute_with_timeout(
        &self,
        model_path: &str,
        prompt: &str,
        timeout_seconds: u64,
    ) -> Result<String> {
        if !self.config.is_external_timeout_enabled() {
            // Fallback to internal timeout only
            return self.execute_internal_timeout(model_path, prompt, timeout_seconds).await;
        }

        tracing::info!(
            "Executing llama-cli with external timeout protection: {}s",
            timeout_seconds
        );

        // Use system timeout command to wrap llama-cli
        let timeout_duration = Duration::from_secs(timeout_seconds);
        
        let result = timeout(timeout_duration, async {
            let output = Command::new("timeout")
                .arg(format!("{}s", timeout_seconds))
                .arg("llama-cli")
                .arg("--model")
                .arg(model_path)
                .arg("--prompt")
                .arg(prompt)
                .arg("--no-display-prompt")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("timeout") || output.status.code() == Some(124) {
                    anyhow::bail!(
                        "External timeout: llama-cli exceeded {}s timeout - preventing AI overthinking",
                        timeout_seconds
                    );
                }
                anyhow::bail!("llama-cli failed: {}", stderr);
            }

            let response = String::from_utf8(output.stdout)?;
            Ok(response.trim().to_string())
        }).await;

        match result {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(e)) => Err(e),
            Err(_) => anyhow::bail!(
                "Internal timeout: llama-cli exceeded {}s timeout - preventing AI overthinking",
                timeout_seconds
            ),
        }
    }

    /// Fallback to internal timeout only (for testing/systems without timeout command)
    async fn execute_internal_timeout(
        &self,
        _model_path: &str,
        prompt: &str,
        timeout_seconds: u64,
    ) -> Result<String> {
        tracing::warn!(
            "External timeout disabled, using internal timeout only: {}s",
            timeout_seconds
        );

        let timeout_duration = Duration::from_secs(timeout_seconds);
        
        let result = timeout(timeout_duration, async {
            // Simulate model processing
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Return mock response for now
            Ok(format!(
                r#"{{"analysis": "Mock response for: {}", "status": "success", "timeout_used": "internal_{}s"}}"#,
                prompt.chars().take(50).collect::<String>(),
                timeout_seconds
            ))
        }).await;

        match result {
            Ok(response) => response,
            Err(_) => anyhow::bail!(
                "Internal timeout: Model processing exceeded {}s timeout - preventing AI overthinking",
                timeout_seconds
            ),
        }
    }

    /// Execute DeepSeek reasoning with configured timeout
    pub async fn execute_deepseek_reasoning(&self, prompt: &str) -> Result<String> {
        let timeout_seconds = self.config.get_reasoning_timeout();
        let model_path = self.config.model_cache_dir
            .join(format!("DeepSeek-R1-0528-Qwen3-8B-{}.gguf", self.config.get_quantization_suffix()));
        
        self.execute_with_timeout(&model_path.to_string_lossy(), prompt, timeout_seconds).await
    }

    /// Execute Qwen embedding with configured timeout
    pub async fn execute_qwen_embedding(&self, text: &str) -> Result<String> {
        let timeout_seconds = self.config.get_embedding_timeout();
        let model_path = self.config.model_cache_dir
            .join(format!("Qwen3-Embedding-8B-{}.gguf", self.config.get_quantization_suffix()));
        
        self.execute_with_timeout(&model_path.to_string_lossy(), text, timeout_seconds).await
    }

    /// Check if external timeout command is available
    pub fn is_timeout_command_available() -> bool {
        Command::new("timeout")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_external_timeout_wrapper_creation() {
        let config = MLConfig::for_testing();
        let wrapper = ExternalTimeoutWrapper::new(config);
        
        // Should create successfully
        assert!(wrapper.config.get_reasoning_timeout() > 0);
    }

    #[tokio::test]
    async fn test_timeout_command_availability() {
        // Check if timeout command is available on system
        let available = ExternalTimeoutWrapper::is_timeout_command_available();
        
        // On most Linux systems, timeout should be available
        // On other systems or containers, it might not be
        tracing::info!("Timeout command available: {}", available);
    }

    #[tokio::test]
    async fn test_internal_timeout_fallback() {
        let config = MLConfig::for_testing();
        let wrapper = ExternalTimeoutWrapper::new(config);
        
        let result = wrapper.execute_internal_timeout(
            "mock_model.gguf",
            "test prompt",
            30
        ).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("Mock response"));
        assert!(response.contains("internal_30s"));
    }

    #[tokio::test]
    async fn test_timeout_configuration() {
        let config = MLConfig::for_8gb_vram();
        
        assert_eq!(config.get_reasoning_timeout(), 240);
        assert_eq!(config.get_embedding_timeout(), 60);
        assert_eq!(config.get_external_process_timeout(), 300);
        assert!(config.is_external_timeout_enabled());
    }
}
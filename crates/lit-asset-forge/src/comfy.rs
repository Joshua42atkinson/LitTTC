// comfy.rs — ComfyUI API client for LongCat image generation

use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, warn};

/// High-level ComfyUI client.
#[derive(Debug, Clone)]
pub struct ComfyClient {
    client: Client,
    base_url: String,
}

impl ComfyClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .expect("reqwest client build"),
            base_url: base_url.into(),
        }
    }

    /// Health check via ComfyUI's system_stats endpoint.
    pub async fn is_healthy(&self) -> bool {
        self.client
            .get(format!("{}/system_stats", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Wait until ComfyUI responds, up to `max_wait`.
    pub async fn wait_for_ready(&self, max_wait: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        let mut attempt = 0;
        loop {
            if self.is_healthy().await {
                info!("ComfyUI ready after {:?}", start.elapsed());
                return Ok(());
            }
            if start.elapsed() > max_wait {
                return Err(anyhow!("ComfyUI did not become ready in {:?}", max_wait));
            }
            attempt += 1;
            tokio::time::sleep(Duration::from_secs(2)).await;
            if attempt % 5 == 0 {
                info!("Still waiting for ComfyUI... ({:?})", start.elapsed());
            }
        }
    }

    /// Submit a LongCat text-to-image workflow and return the prompt_id.
    #[allow(clippy::too_many_arguments)]
    pub async fn submit_longcat_workflow(
        &self,
        model: &str,
        prompt: &str,
        negative_prompt: &str,
        width: u32,
        height: u32,
        steps: u32,
        guidance_scale: f32,
        seed: u64,
        filename_prefix: &str,
    ) -> Result<String> {
        let workflow = build_longcat_workflow(
            model,
            prompt,
            negative_prompt,
            width,
            height,
            steps,
            guidance_scale,
            seed,
            filename_prefix,
        );

        let payload = json!({
            "prompt": workflow,
            "client_id": format!("lit-asset-forge-{}", uuid::Uuid::new_v4()),
        });

        let response = self
            .client
            .post(format!("{}/prompt", self.base_url))
            .json(&payload)
            .send()
            .await
            .context("Failed to submit ComfyUI prompt")?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("ComfyUI /prompt failed: {}", body));
        }

        let result: PromptSubmitResponse = response
            .json()
            .await
            .context("Failed to parse ComfyUI prompt response")?;

        if let Some(id) = result.prompt_id {
            info!("Submitted ComfyUI prompt_id: {}", id);
            Ok(id)
        } else {
            Err(anyhow!("ComfyUI response missing prompt_id: {:?}", result))
        }
    }

    /// Poll ComfyUI history until the prompt completes, then return the saved filename.
    pub async fn wait_for_output(
        &self,
        prompt_id: &str,
        poll_interval: Duration,
        max_wait: Duration,
    ) -> Result<ComfyOutput> {
        let start = std::time::Instant::now();
        loop {
            if start.elapsed() > max_wait {
                return Err(anyhow!("ComfyUI generation timed out after {:?}", max_wait));
            }

            let response = self
                .client
                .get(format!("{}/history/{}", self.base_url, prompt_id))
                .send()
                .await
                .context("Failed to poll ComfyUI history")?;

            if response.status().is_success() {
                let history: serde_json::Value = response
                    .json()
                    .await
                    .context("Failed to parse ComfyUI history")?;

                if let Some(entry) = history.get(prompt_id) {
                    // Check for errors first
                    if let Some(errors) = entry.get("outputs_error") {
                        warn!("ComfyUI errors: {:?}", errors);
                    }

                    if let Some(outputs) = entry.get("outputs") {
                        for (_node_id, output) in outputs.as_object().unwrap_or(&serde_json::Map::new()) {
                            if let Some(images) = output.get("images").and_then(|v| v.as_array()) {
                                if let Some(img) = images.first() {
                                    let filename = img
                                        .get("filename")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let subfolder = img
                                        .get("subfolder")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let comfy_type = img
                                        .get("type")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("output")
                                        .to_string();

                                    if !filename.is_empty() {
                                        return Ok(ComfyOutput {
                                            filename,
                                            subfolder,
                                            comfy_type,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Resolve the on-disk path of a ComfyUI output file.
    pub fn resolve_output_path(&self, output: &ComfyOutput, comfy_home: &str) -> PathBuf {
        let mut path = PathBuf::from(comfy_home);
        path.push("output");
        if !output.subfolder.is_empty() {
            path.push(&output.subfolder);
        }
        path.push(&output.filename);
        path
    }
}

#[derive(Debug, Clone, Deserialize)]
struct PromptSubmitResponse {
    prompt_id: Option<String>,
    #[allow(dead_code)]
    number: Option<u32>,
    #[allow(dead_code)]
    node_errors: Option<serde_json::Value>,
}

/// Output metadata returned by ComfyUI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfyOutput {
    pub filename: String,
    pub subfolder: String,
    pub comfy_type: String,
}

/// Build the JSON node graph for LongCat-Image text-to-image.
#[allow(clippy::too_many_arguments)]
fn build_longcat_workflow(
    model: &str,
    prompt: &str,
    negative_prompt: &str,
    width: u32,
    height: u32,
    steps: u32,
    guidance_scale: f32,
    seed: u64,
    filename_prefix: &str,
) -> serde_json::Value {
    json!({
        "1": {
            "class_type": "LongCatImageModelLoader",
            "inputs": {
                "model_path": model,
                "dtype": "bfloat16",
                "enable_cpu_offload": "true",
                "attention_backend": "default"
            }
        },
        "2": {
            "class_type": "LongCatImageTextToImage",
            "inputs": {
                "longcat_pipeline": ["1", 0],
                "prompt": prompt,
                "negative_prompt": negative_prompt,
                "width": width,
                "height": height,
                "steps": steps,
                "guidance_scale": guidance_scale,
                "seed": seed,
                "enable_cfg_renorm": "true",
                "enable_prompt_rewrite": "true"
            }
        },
        "3": {
            "class_type": "SaveImage",
            "inputs": {
                "images": ["2", 0],
                "filename_prefix": filename_prefix
            }
        }
    })
}

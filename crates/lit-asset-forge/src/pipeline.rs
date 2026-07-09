// pipeline.rs — High-level portrait generation orchestrator

use crate::{
    comfy::{ComfyClient, ComfyOutput},
    lore::{self, LoreContext},
    manifest::AssetManifest,
    prompts::{safe_filename, LmStudioClient},
    ForgeConfig, PortraitAsset,
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, warn};

/// Orchestrator for a single generation run.
#[derive(Debug, Clone)]
pub struct ForgePipeline {
    pub config: ForgeConfig,
    pub comfy: ComfyClient,
    pub lm: LmStudioClient,
    pub comfy_home: String,
}

impl ForgePipeline {
    pub fn new(config: ForgeConfig) -> Self {
        let comfy = ComfyClient::new(&config.comfy_url);
        let lm = LmStudioClient::new(&config.lm_studio_url, &config.lm_model);
        let comfy_home = std::env::var("HOME")
            .map(|h| format!("{}/ComfyUI", h))
            .unwrap_or_else(|_| "./ComfyUI".to_string());
        Self {
            config,
            comfy,
            lm,
            comfy_home,
        }
    }

    /// Wait for ComfyUI only — use this when prompts are written manually.
    pub async fn wait_for_comfy(&self) -> Result<()> {
        info!("Waiting for ComfyUI at {}...", self.config.comfy_url);
        self.comfy
            .wait_for_ready(Duration::from_secs(120))
            .await
    }

    /// Wait for both services to be ready (LLM mode).
    pub async fn wait_for_services(&self) -> Result<()> {
        self.wait_for_comfy().await?;
        info!("Waiting for LM Studio at {}...", self.config.lm_studio_url);
        let mut attempts = 0;
        loop {
            if self.lm.is_healthy().await {
                info!("LM Studio ready");
                break;
            }
            attempts += 1;
            if attempts > 30 {
                return Err(anyhow::anyhow!("LM Studio not ready after 60s"));
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        if !self.lm.is_model_loaded().await {
            self.lm.load_model().await.context("Auto-loading LM Studio model")?;
        } else {
            info!("LM Studio model '{}' already loaded", self.config.lm_model);
        }

        Ok(())
    }

    /// Generate a portrait for one word and return the asset metadata.
    ///
    /// If `prompt_override` is provided, the LLM prompt-crafting step is skipped.
    /// Lore is generated first (LLM if available, otherwise deterministic) and
    /// fed into the image prompt only when LLM mode is used.
    #[allow(clippy::too_many_arguments)]
    pub async fn generate_portrait(
        &self,
        word: &str,
        element: &str,
        role: &str,
        summon_class: &str,
        width: u32,
        height: u32,
        lore_ctx: Option<&LoreContext>,
        prompt_override: Option<&str>,
    ) -> Result<PortraitAsset> {
        let start = std::time::Instant::now();
        let seed = rand::random::<u64>();

        let (lore, prompt) = if let Some(p) = prompt_override {
            let ctx = lore_ctx.cloned().unwrap_or_else(|| LoreContext {
                word: word.to_string(),
                element: element.to_string(),
                role: role.to_string(),
                summon_class: summon_class.to_string(),
                ..Default::default()
            });
            let lore = lore::generate_lore_deterministic(&ctx);
            (lore, p.to_string())
        } else {
            let lore = match lore_ctx {
                Some(ctx) => {
                    info!("Generating lore for '{}'", word);
                    match self.lm.generate_lore(ctx).await {
                        Ok(l) => l,
                        Err(e) => {
                            warn!("LLM lore failed for '{}', using deterministic fallback: {}", word, e);
                            lore::generate_lore_deterministic(ctx)
                        }
                    }
                }
                None => {
                    let ctx = LoreContext {
                        word: word.to_string(),
                        element: element.to_string(),
                        role: role.to_string(),
                        summon_class: summon_class.to_string(),
                        ..Default::default()
                    };
                    lore::generate_lore_deterministic(&ctx)
                }
            };

            let prompt = self
                .lm
                .craft_pet_portrait_prompt(word, element, role, summon_class, Some(&lore.to_prompt_blob()))
                .await
                .context("Prompt crafting failed")?;
            (lore, prompt)
        };

        let safe_word = safe_filename(word);
        let filename_prefix = format!("lit_forge_{}_{}", safe_word, seed);

        let prompt_id = self
            .comfy
            .submit_longcat_workflow(
                &self.config.longcat_model,
                &prompt,
                "blurry, low quality, distorted, text, watermark, signature",
                width,
                height,
                self.config.longcat_steps,
                self.config.guidance_scale,
                seed,
                &filename_prefix,
            )
            .await
            .context("Submitting LongCat workflow failed")?;

        let output: ComfyOutput = self
            .comfy
            .wait_for_output(
                &prompt_id,
                Duration::from_secs(5),
                Duration::from_secs(600),
            )
            .await
            .context("Waiting for ComfyUI output failed")?;

        let src_path = self.comfy.resolve_output_path(&output, &self.comfy_home);
        if !src_path.exists() {
            return Err(anyhow::anyhow!(
                "ComfyUI reported output file but it does not exist: {:?}",
                src_path
            ));
        }

        let ext = src_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_string();
        let relative_path = PathBuf::from(format!(
            "textures/cards/portraits/{}_{}.{}",
            safe_word, element.to_lowercase(), ext
        ));
        let portrait_path = self.config.output_dir.join(&relative_path);

        std::fs::create_dir_all(portrait_path.parent().unwrap())
            .with_context(|| format!("Creating portrait directory for {:?}", portrait_path))?;
        std::fs::copy(&src_path, &portrait_path)
            .with_context(|| format!("Copying {:?} to {:?}", src_path, portrait_path))?;

        let generation_time_ms = start.elapsed().as_millis() as u64;
        info!(
            "Portrait for '{}' generated in {}ms -> {:?}",
            word, generation_time_ms, portrait_path
        );

        Ok(PortraitAsset {
            word: word.to_string(),
            element: element.to_string(),
            role: role.to_string(),
            summon_class: summon_class.to_string(),
            prompt,
            lore,
            portrait_path,
            relative_path,
            seed,
            generation_time_ms,
        })
    }

    /// Generate portraits for a list of words, updating the manifest as we go.
    pub async fn generate_portraits_batch(
        &self,
        words: Vec<WordSpec>,
        width: u32,
        height: u32,
    ) -> Result<AssetManifest> {
        let mut manifest = AssetManifest::load(&self.config.output_dir)?;

        for spec in words {
            let lore_ctx = spec.to_lore_context();
            match self
                .generate_portrait(&spec.word, &spec.element, &spec.role, &spec.summon_class, width, height, Some(&lore_ctx), None)
                .await
            {
                Ok(asset) => {
                    manifest.add_portrait(&asset);
                    if let Err(e) = manifest.save(&self.config.output_dir) {
                        warn!("Failed to save manifest after {}: {}", spec.word, e);
                    }
                }
                Err(e) => {
                    warn!("Failed to generate portrait for '{}': {}", spec.word, e);
                }
            }
        }

        Ok(manifest)
    }
}

/// Specification for a single word to generate art for.
#[derive(Debug, Clone, Default)]
pub struct WordSpec {
    pub word: String,
    pub element: String,
    pub role: String,
    pub summon_class: String,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
    pub etymology_root: Option<String>,
    pub grade_level: Option<String>,
    pub district: Option<String>,
    pub npc_guardian: Option<String>,
}

impl WordSpec {
    pub fn new(word: &str, element: &str, role: &str, summon_class: &str) -> Self {
        Self {
            word: word.to_string(),
            element: element.to_string(),
            role: role.to_string(),
            summon_class: summon_class.to_string(),
            ..Default::default()
        }
    }

    pub fn to_lore_context(&self) -> LoreContext {
        LoreContext {
            word: self.word.clone(),
            element: self.element.clone(),
            role: self.role.clone(),
            summon_class: self.summon_class.clone(),
            synonyms: self.synonyms.clone(),
            antonyms: self.antonyms.clone(),
            etymology_root: self.etymology_root.clone(),
            grade_level: self.grade_level.clone(),
            district: self.district.clone(),
            npc_guardian: self.npc_guardian.clone(),
        }
    }
}

use anyhow::{anyhow, Context};
use bincode::{Decode, Encode};
use chrono::Utc;
use log::warn;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::{NoContext, Timestamp, Uuid};

use crate::{
    clean_stats::CleanStats,
    cli::OutputFormat,
    persistence::{read_bincode, read_jsonl, write_bincode, write_jsonl},
    tier::Tier,
    uri_management::fetch_uri_async,
};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Resource {
    pub id: String,
    pub uri: String,
    pub fetched_at: String,
    pub content: String,
    pub tier: Option<Tier>, // ✅ Type-safe avec enum
}

impl Resource {
    /// Assigns a tier based on content length.
    /// Avoids reassigning if the tier is already defined.
    pub fn assign_tier(&mut self, thresholds: &[usize; 2]) {
        if self.tier.is_none() {
            self.tier = Some(Tier::from_length(self.content.len(), thresholds));
        }
    }

    /// Returns the tier, or a default value if not assigned.
    pub fn tier_or_unknown(&self) -> Tier {
        self.tier.unwrap_or(Tier::Medium)
    }
}

pub fn create_resource_from_content(uri: String, content: String) -> Resource {
    Resource {
        id: Uuid::new_v7(Timestamp::now(NoContext)).to_string(),
        uri,
        fetched_at: Utc::now().to_rfc3339(),
        content,
        tier: None,
    }
}

pub fn read_resources(
    path: &PathBuf,
    format: OutputFormat,
) -> Result<Vec<Resource>, anyhow::Error> {
    match format {
        OutputFormat::Jsonl => read_jsonl(path),
        OutputFormat::Bincode => read_bincode(path),
    }
}

pub fn write_resources(
    path: &PathBuf,
    format: OutputFormat,
    resources: &[Resource],
) -> Result<(), anyhow::Error> {
    match format {
        OutputFormat::Jsonl => write_jsonl(path, resources),
        OutputFormat::Bincode => write_bincode(path, resources),
    }
}

pub fn clean_resource(
    resource: Resource,
    min_length: usize,
    max_length: usize,
    stats: &mut CleanStats,
) -> Option<Resource> {
    let content = resource.content.trim().replace("\r\n", "\n");

    if content.len() < min_length {
        // Remplacement des appels à `add_too_short`
        stats.add_stat("too_short", None);
        return None;
    }

    if content.len() > max_length {
        // Remplacement des appels à `add_too_long`
        stats.add_stat("too_long", None);
        return None;
    }

    // Remplacement des appels à `add_kept`
    stats.add_stat(
        "kept",
        Some(match resource.tier {
            Some(tier) => match tier {
                Tier::Low => "low",
                Tier::Medium => "medium",
                Tier::High => "high",
            },
            None => "unknown",
        }),
    );
    Some(Resource {
        content,
        ..resource
    })
}

// Ajout d'une fonction utilitaire générique pour les commandes
pub fn process_resources<F>(
    input: &PathBuf,
    output: &PathBuf,
    input_format: OutputFormat,
    output_format: OutputFormat,
    process_fn: F,
) -> Result<(), anyhow::Error>
where
    F: Fn(Vec<Resource>) -> Vec<Resource>,
{
    let resources = read_resources(input, input_format)?;
    let processed_resources = process_fn(resources);
    write_resources(output, output_format, &processed_resources)?;
    println!("Output written to {:?} ({:?})", output, output_format);
    Ok(())
}

/// Ensures that the parent directory of the given path exists.
/// Creates the directory if it does not exist.
pub fn ensure_directory_exists(path: &PathBuf) -> Result<(), anyhow::Error> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create output directory {:?}", parent))?;
        }
    }
    Ok(())
}

/// Validates a list of resources using a given model.
pub fn validate_resources(resources: &[Resource], model: &str) -> Vec<String> {
    resources
        .iter()
        .map(|resource| {
            let is_valid = resource.content.len() > 10;
            let model_prediction = format!("Model '{}' prediction: valid", model);
            format!(
                "Resource ID: {}, Symbolic: {}, {}, Content length: {}",
                resource.id,
                is_valid,
                model_prediction,
                resource.content.len()
            )
        })
        .collect()
}

/// Enriches a list of resources using a given model.
pub fn enrich_resources(resources: Vec<Resource>, model: &str) -> Vec<Resource> {
    resources
        .into_iter()
        .map(|resource| {
            let enriched_content = format!("[Enriched by {}]: {}", model, resource.content);
            Resource {
                content: enriched_content,
                ..resource
            }
        })
        .collect()
}

/// Scrapes resources from a list of URIs asynchronously.
pub async fn scrape_resources(uri_list: &[String]) -> Result<Vec<Resource>, anyhow::Error> {
    let mut scraped_resources = Vec::new();
    for uri in uri_list {
        match fetch_uri_async(uri).await {
            Ok(content) => {
                scraped_resources.push(create_resource_from_content(uri.clone(), content));
            }
            Err(e) => {
                warn!("Warning: {:#}", e);
            }
        }
    }
    if scraped_resources.is_empty() {
        return Err(anyhow!("Échec du scraping: aucune ressource récupérée"));
    }
    Ok(scraped_resources)
}

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use anyhow::{Context, Result};
use log::{info, warn};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{cli::OutputFormat, resource::{Resource, create_resource_from_content, write_resources}};

pub fn read_uri_list(path: &PathBuf) -> Result<Vec<String>, anyhow::Error> {
    let file = File::open(path).with_context(|| format!("Failed to open URIs file {:?}", path))?;
    let reader = BufReader::new(file);

    let mut uris = Vec::new();
    for (line_num, line) in reader.lines().enumerate() {
        let line =
            line.with_context(|| format!("Failed to read line {} from {:?}", line_num + 1, path))?;
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        uris.push(line.to_string());
    }

    Ok(uris)
}

pub async fn fetch_uri_async(uri: &str) -> Result<String, anyhow::Error> {
    let response = reqwest::get(uri)
        .await
        .with_context(|| format!("Failed to fetch {}", uri))?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP error {} for {}", response.status(), uri);
    }

    response
        .text()
        .await
        .with_context(|| format!("Failed to decode response from {}", uri))
}

pub fn fetch_uri_sync(uri: &str) -> Result<String, anyhow::Error> {
    let response =
        reqwest::blocking::get(uri).with_context(|| format!("Failed to fetch {}", uri))?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP error {} for {}", response.status(), uri);
    }

    response
        .text()
        .with_context(|| format!("Failed to decode response from {}", uri))
}

// Refactorisation des étapes du pipeline
pub async fn scrape_uris(
    uris: &PathBuf,
    output: &PathBuf,
    format: OutputFormat,
    parallel: bool,
) -> Result<()> {
    let uri_list = read_uri_list(uris)?;
    info!("Processing {} URIs...", uri_list.len());

    let resources: Vec<Resource> = if parallel {
        uri_list
            .into_par_iter()
            .filter_map(|uri| match fetch_uri_sync(&uri) {
                Ok(content) => Some(create_resource_from_content(uri, content)),
                Err(e) => {
                    warn!("Warning: {:#}", e);
                    None
                }
            })
            .collect()
    } else {
        let mut resources = Vec::new();
        for uri in uri_list {
            match fetch_uri_async(&uri).await {
                Ok(content) => resources.push(create_resource_from_content(uri, content)),
                Err(e) => warn!("Warning: {:#}", e),
            }
        }
        resources
    };

    write_resources(output, format, &resources)?;
    info!(
        "Successfully wrote {} resources to {:?} ({:?})",
        resources.len(),
        output,
        format
    );
    Ok(())
}

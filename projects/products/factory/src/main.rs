use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use factory::{
    clean_stats::CleanStats,
    cli::{Cli, Commands, OutputFormat},
    persistence::{read_bincode, read_jsonl, write_bincode, write_jsonl},
    resource::Resource,
};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};
use uuid::{NoContext, Timestamp, Uuid};

fn read_uri_list(path: &PathBuf) -> Result<Vec<String>> {
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

fn read_content_from_uri(uri: &str) -> Result<String> {
    let path_str = uri.strip_prefix("file://").unwrap_or(uri);
    let path = PathBuf::from(path_str);

    anyhow::ensure!(path.exists(), "File does not exist: {:?}", path);

    let bytes = std::fs::read(&path).with_context(|| format!("Failed to read file {:?}", path))?;

    String::from_utf8(bytes).with_context(|| format!("File contains invalid UTF-8: {:?}", path))
}

fn create_resource(uri: String) -> Result<Resource> {
    let timestamp = Timestamp::now(NoContext);
    let content = read_content_from_uri(&uri)?;

    Ok(Resource {
        id: Uuid::new_v7(timestamp).to_string(),
        uri,
        fetched_at: Utc::now().to_rfc3339(),
        content,
        tier: None, // Initialiser le palier à None
    })
}

fn read_resources(path: &PathBuf, format: OutputFormat) -> Result<Vec<Resource>> {
    match format {
        OutputFormat::Jsonl => read_jsonl(path),
        OutputFormat::Bincode => read_bincode(path),
    }
}

fn write_resources(path: &PathBuf, format: OutputFormat, resources: &[Resource]) -> Result<()> {
    match format {
        OutputFormat::Jsonl => write_jsonl(path, resources),
        OutputFormat::Bincode => write_bincode(path, resources),
    }
}

fn clean_resource(
    resource: Resource,
    min_length: usize,
    max_length: usize,
    stats: &mut CleanStats,
) -> Option<Resource> {
    let content = resource.content.trim().replace("\r\n", "\n");

    if content.len() < min_length {
        stats.add_too_short();
        return None;
    }

    if content.len() > max_length {
        stats.add_too_long();
        return None;
    }

    stats.add_kept(resource.tier.as_deref().unwrap_or("unknown"));
    Some(Resource {
        content,
        ..resource
    })
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("Thresholds: {:?}", cli.thresholds); // Debug: Afficher les seuils

    match cli.cmd {
        Commands::Scrape {
            uris,
            output,
            format,
            parallel,
        } => {
            let uri_list = read_uri_list(&uris)?;
            println!("Processing {} URIs...", uri_list.len());

            let resources: Vec<Resource> = if parallel {
                uri_list
                    .into_par_iter()
                    .filter_map(|uri| match create_resource(uri.clone()) {
                        Ok(mut resource) => {
                            resource.assign_tier(&cli.thresholds); // Assigner le palier
                            Some(resource)
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to process {}: {}", uri, e);
                            None
                        }
                    })
                    .collect()
            } else {
                let mut resources = Vec::new();
                for uri in uri_list {
                    match create_resource(uri.clone()) {
                        Ok(mut resource) => {
                            resource.assign_tier(&cli.thresholds); // Assigner le palier
                            resources.push(resource)
                        }
                        Err(e) => eprintln!("Warning: Failed to process {}: {}", uri, e),
                    }
                }
                resources
            };

            write_resources(&output, format, &resources)?;
            println!(
                "Successfully wrote {} resources to {:?} ({:?})",
                resources.len(),
                output,
                format
            );
        }
        Commands::Clean {
            input,
            input_format,
            output,
            output_format,
            min_length,
            max_length,
        } => {
            let resources = read_resources(&input, input_format)?;
            let mut stats = CleanStats::new();

            let cleaned: Vec<Resource> = resources
                .into_iter()
                .filter_map(|resource| clean_resource(resource, min_length, max_length, &mut stats))
                .collect();

            write_resources(&output, output_format, &cleaned)?;
            println!("{}", stats);
            println!("Output written to {:?} ({:?})", output, output_format);
        }
    }

    Ok(())
}

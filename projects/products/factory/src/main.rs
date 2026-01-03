use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use factory::{
    clean_stats::CleanStats,
    cli::{Cli, Commands, OutputFormat},
    resource::{
        Resource, clean_resource, enrich_resources, ensure_directory_exists, process_resources, read_resources, scrape_resources, validate_resources, write_resources
    },
    uri_management::{read_uri_list, scrape_uris},
};
use log::{info};
use std::cell::RefCell;
use std::rc::Rc;
use uuid::{NoContext, Timestamp, Uuid};

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    let cli = Cli::parse();

    match cli.cmd {
        Commands::Scrape {
            uris,
            output,
            format,
            parallel,
        } => {
            scrape_uris(&uris, &output, format, parallel).await?;
        }
        Commands::Clean {
            input,
            input_format,
            output,
            output_format,
            min_length,
            max_length,
        } => {
            // Réintégration de `stats` dans l'appel à `process_resources`
            let stats = Rc::new(RefCell::new(CleanStats::new()));
            process_resources(&input, &output, input_format, output_format, {
                let stats = Rc::clone(&stats);
                move |resources| {
                    resources
                        .into_iter()
                        .filter_map(|resource| {
                            clean_resource(
                                resource,
                                min_length,
                                max_length,
                                &mut stats.borrow_mut(),
                            )
                        })
                        .collect()
                }
            })?;
            info!("Cleaning completed. Stats: {}", stats.borrow());
        }
        Commands::Filter {
            input,
            output,
            tier,
        } => {
            process_resources(
                &input,
                &output,
                OutputFormat::Jsonl,
                OutputFormat::Jsonl,
                |resources| {
                    resources
                        .into_iter()
                        .filter(
                            |r| matches!(r.tier, Some(ref _tier) if format!("{:?}", _tier) == tier),
                        )
                        .collect()
                },
            )?;
        }
        Commands::Enrich {
            input,
            output,
            model,
            batch_size,
        } => {
            process_resources(
                &input,
                &output,
                OutputFormat::Jsonl,
                OutputFormat::Jsonl,
                |resources| {
                    resources
                        .chunks(batch_size)
                        .flat_map(|batch| {
                            batch.iter().map(|resource| {
                                let enriched_content =
                                    format!("[Enriched by {}]: {}", model, resource.content);

                                Resource {
                                    content: enriched_content,
                                    ..resource.clone()
                                }
                            })
                        })
                        .collect()
                },
            )?;
        }
        Commands::AutoScrape {
            model,
            output,
            format,
            count,
        } => {
            info!(
                "Starting automated scraping with model '{}' for {} items...",
                model, count
            );

            let scraped_resources: Vec<Resource> = (0..count)
                .map(|i| Resource {
                    id: Uuid::new_v7(Timestamp::now(NoContext)).to_string(),
                    uri: format!("http://example.com/resource/{}", i),
                    fetched_at: Utc::now().to_rfc3339(),
                    content: format!("Generated content {} by model {}", i, model),
                    tier: None,
                })
                .collect();

            write_resources(&output, format, &scraped_resources)?;
            info!("Scraped resources written to {:?} ({:?})", output, format);
        }
        Commands::ValidateDataset {
            input,
            model,
            report,
        } => {
            info!(
                "Validating dataset from {:?} using model '{}'...",
                input, model
            );

            let resources = read_resources(&input, OutputFormat::Jsonl)?;

            let validation_results: Vec<String> = resources
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
                .collect();

            std::fs::write(&report, validation_results.join("\n"))
                .with_context(|| format!("Failed to write validation report to {:?}", report))?;

            info!("Validation report written to {:?}", report);
        }
        Commands::RunPipeline {
            uris,
            output_dir,
            enrich_model,
            validate_model,
        } => {
            info!("Starting pipeline with URIs from {:?}...", uris);

            // Step 1: Scrape
            let scrape_output = output_dir.join("scraped.jsonl");
            ensure_directory_exists(&scrape_output)?;

            let uri_list = read_uri_list(&uris)?;
            let scraped_resources = scrape_resources(&uri_list).await?;
            write_resources(&scrape_output, OutputFormat::Jsonl, &scraped_resources)?;
            info!("Scraping completed. Output: {:?}", scrape_output);

            let clean_output = output_dir.join("cleaned.jsonl");
            let mut stats = CleanStats::new();
            let cleaned_resources: Vec<Resource> = scraped_resources
                .into_iter()
                .filter_map(|resource| clean_resource(resource, 200, 2_000_000, &mut stats))
                .collect();
            write_resources(&clean_output, OutputFormat::Jsonl, &cleaned_resources)?;
            info!("Cleaning completed. Output: {:?}", clean_output);
            info!("{}", stats);

            // Step 3: Enrich
            let enrich_output = output_dir.join("enriched.jsonl");
            let enriched_resources = enrich_resources(cleaned_resources, &enrich_model);
            write_resources(&enrich_output, OutputFormat::Jsonl, &enriched_resources)?;
            info!("Enrichment completed. Output: {:?}", enrich_output);

            // Step 4: Validate
            let validate_report = output_dir.join("validation_report.txt");
            let validation_results = validate_resources(&enriched_resources, &validate_model);
            std::fs::write(&validate_report, validation_results.join("\n")).with_context(|| {
                format!("Failed to write validation report to {:?}", validate_report)
            })?;
            info!("Validation completed. Report: {:?}", validate_report);
        }
        Commands::Serve { port, dir } => {
            if !dir.exists() {
                std::fs::create_dir_all(&dir)
                    .with_context(|| format!("Failed to create directory {:?}", dir))?;
            }

            let files = warp::fs::dir(dir.clone());

            info!(
                "Server running on http://localhost:{} serving {:?}",
                port, dir
            );
            warp::serve(files).run(([127, 0, 0, 1], port)).await;
        }
    }

    Ok(())
}

fn init_logging() {
    env_logger::init();
}

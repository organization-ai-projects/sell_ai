use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Dataset factory - ingest, clean, and process resources
#[derive(Parser)]
#[command(name = "factory")]
#[command(about = "Minimal dataset factory - ingest URIs and process resources")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,

    /// Thresholds for resource tiers (low, high)
    #[arg(long, value_delimiter = ',', default_value = "100,1000")]
    pub thresholds: Vec<usize>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Read a list of URIs and write raw resources
    Scrape {
        /// Text file containing URIs (one per line). Lines starting with # are ignored.
        #[arg(long)]
        uris: PathBuf,

        /// Output file path
        #[arg(long)]
        output: PathBuf,

        /// Output format
        #[arg(long, default_value = "jsonl")]
        format: OutputFormat,

        /// Enable parallel processing
        #[arg(long)]
        parallel: bool,
    },
    /// Clean resources by trimming, normalizing, and filtering
    Clean {
        /// Input file path
        #[arg(long)]
        input: PathBuf,

        /// Input format
        #[arg(long, default_value = "jsonl")]
        input_format: OutputFormat,

        /// Output file path
        #[arg(long)]
        output: PathBuf,

        /// Output format
        #[arg(long, default_value = "jsonl")]
        output_format: OutputFormat,

        /// Minimum content length
        #[arg(long, default_value_t = 200)]
        min_length: usize,

        /// Maximum content length
        #[arg(long, default_value_t = 2_000_000)]
        max_length: usize,
    },
    /// Filter resources by tier
    Filter {
        /// Input file path
        #[arg(long)]
        input: PathBuf,

        /// Output file path
        #[arg(long)]
        output: PathBuf,

        /// Tier to filter by (low, medium, high)
        #[arg(long)]
        tier: String,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Jsonl,
    Bincode,
}

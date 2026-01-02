# Factory

[![CI](https://github.com/TON-USERNAME/factory/actions/workflows/ci.yml/badge.svg)](https://github.com/TON-USERNAME/factory/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/TON-USERNAME/factory/branch/main/graph/badge.svg)](https://codecov.io/gh/TON-USERNAME/factory)
[![dependency status](https://deps.rs/repo/github/TON-USERNAME/factory/status.svg)](https://deps.rs/repo/github/TON-USERNAME/factory)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Dataset factory - ingest, clean, and process resources.

## Features

- 🚀 Fast parallel scraping with Rayon
- 📦 Multiple output formats (JSONL, Bincode)
- 🧹 Robust cleaning and validation
- 🔒 Security-first design

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Scrape URIs
factory scrape --uris uris.txt --output raw.jsonl --format jsonl

# Clean data
factory clean --input raw.jsonl --output clean.jsonl --min-length 200
```

## Development

```bash
# Run tests
cargo test

# Check code quality
cargo clippy

# Format code
cargo fmt
```

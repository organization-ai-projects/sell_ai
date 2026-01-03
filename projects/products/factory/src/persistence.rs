use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

use anyhow::Context;
use bincode::Encode;
use serde::{Deserialize, Serialize};

//jsonl functions
pub fn read_jsonl<T: for<'de> Deserialize<'de>>(path: &PathBuf) -> Result<Vec<T>, anyhow::Error> {
    let file = File::open(path).with_context(|| format!("Failed to open input file {:?}", path))?;
    let reader = BufReader::new(file);

    let mut items = Vec::new();
    for (line_num, line) in reader.lines().enumerate() {
        let line =
            line.with_context(|| format!("Failed to read line {} from {:?}", line_num + 1, path))?;
        let item: T = serde_json::from_str(&line).with_context(|| {
            format!(
                "Failed to parse JSONL line {} from {:?}",
                line_num + 1,
                path
            )
        })?;
        items.push(item);
    }

    Ok(items)
}

pub fn write_jsonl<T: Serialize>(path: &PathBuf, items: &[T]) -> Result<(), anyhow::Error> {
    let file =
        File::create(path).with_context(|| format!("Failed to create output file {:?}", path))?;
    let mut writer = BufWriter::new(file);

    for item in items {
        serde_json::to_writer(&mut writer, item)
            .with_context(|| "Failed to serialize item to JSON")?;
        writer
            .write_all(b"\n")
            .with_context(|| "Failed to write newline")?;
    }

    writer.flush()?;
    Ok(())
}

//bincode functions
pub fn write_bincode<T: Encode>(path: &PathBuf, items: &[T]) -> Result<(), anyhow::Error> {
    let file =
        File::create(path).with_context(|| format!("Failed to create output file {:?}", path))?;
    let mut writer = BufWriter::new(file);

    bincode::encode_into_std_write(items, &mut writer, bincode::config::standard())
        .with_context(|| "Failed to encode bincode data")?;

    writer.flush()?;
    Ok(())
}

pub fn read_bincode<T>(path: &PathBuf) -> Result<Vec<T>, anyhow::Error>
where
    T: bincode::Decode<()>,
{
    let file = File::open(path).with_context(|| format!("Failed to open input file {:?}", path))?;
    let mut buffer = Vec::new();
    BufReader::new(file).read_to_end(&mut buffer)?;

    let config = bincode::config::standard();
    let (decoded, _): (Vec<T>, usize) = bincode::decode_from_slice(&buffer, config)
        .with_context(|| "Failed to decode bincode data")?;

    Ok(decoded)
}

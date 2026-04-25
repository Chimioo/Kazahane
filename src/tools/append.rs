use anyhow::{Context, Result};
use serde_json::Value;
use std::io::Write;
use std::path::Path;

pub fn run(args: &Value, working_dir: &str) -> Result<String> {
    let path_str = args["path"].as_str().unwrap_or("");
    let content = args["content"].as_str().unwrap_or("");

    let path = Path::new(working_dir).join(path_str);

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .with_context(|| format!("Failed to open file for appending: {}", path.display()))?;

    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to append to file: {}", path.display()))?;

    Ok(format!(
        "Appended {} bytes to {}",
        content.len(),
        path_str
    ))
}

use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::Path;

pub fn run(args: &Value, working_dir: &str) -> Result<String> {
    let path_str = args["path"].as_str().unwrap_or("");
    let content = args["content"].as_str().unwrap_or("");

    let path = Path::new(working_dir).join(path_str);

    if path.exists() {
        bail!(
            "File already exists: {}. Use 'edit' or 'append' to modify existing files.",
            path.display()
        );
    }

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directories for: {}", path.display()))?;
    }

    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    Ok(format!(
        "Created file: {} ({} bytes)",
        path_str,
        content.len()
    ))
}

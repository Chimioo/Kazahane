use anyhow::Result;
use serde_json::Value;
use std::path::Path;

pub fn run(args: &Value, working_dir: &str) -> Result<String> {
    let pattern = args["pattern"].as_str().unwrap_or("**/*");

    let base = Path::new(working_dir);
    let full_pattern = base.join(pattern).to_string_lossy().to_string();

    let mut paths: Vec<String> = glob::glob(&full_pattern)?
        .filter_map(|entry| entry.ok())
        .filter(|p| p.is_file())
        .map(|p| {
            // Show relative to working_dir if possible
            p.strip_prefix(base)
                .map(|rel| rel.display().to_string())
                .unwrap_or_else(|_| p.display().to_string())
        })
        .collect();

    paths.sort();

    if paths.is_empty() {
        Ok(format!("No files matched pattern: {pattern}"))
    } else {
        Ok(format!("{} file(s) matched:\n{}", paths.len(), paths.join("\n")))
    }
}

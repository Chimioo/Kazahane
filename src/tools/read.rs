use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;

pub fn run(args: &Value, working_dir: &str) -> Result<String> {
    let path_str = args["path"].as_str().unwrap_or("");
    let start_line = args["start_line"].as_u64().map(|n| n as usize);
    let end_line = args["end_line"].as_u64().map(|n| n as usize);

    let path = Path::new(working_dir).join(path_str);
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();

    // Resolve range (1-indexed from user, 0-indexed internally)
    let start = start_line.map(|n| n.saturating_sub(1)).unwrap_or(0);
    let end = end_line.map(|n| n.min(total)).unwrap_or(total);

    if start >= total {
        return Ok(format!(
            "start_line {sl} is beyond end of file ({total} lines)",
            sl = start_line.unwrap_or(1)
        ));
    }

    let slice = &lines[start..end];
    let width = end.to_string().len();

    let numbered: Vec<String> = slice
        .iter()
        .enumerate()
        .map(|(i, line)| format!("{:>width$} | {}", start + i + 1, line))
        .collect();

    Ok(format!(
        "File: {path_str}  (lines {}-{} of {total})\n{}",
        start + 1,
        end,
        numbered.join("\n")
    ))
}

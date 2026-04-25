use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::Path;

pub fn run(args: &Value, working_dir: &str) -> Result<String> {
    let path_str = args["path"].as_str().unwrap_or("");
    let old_str = args["old_str"].as_str().unwrap_or("");
    let new_str = args["new_str"].as_str().unwrap_or("");
    let occurrence = args["occurrence"].as_u64().map(|n| n as usize);

    let path = Path::new(working_dir).join(path_str);
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    if old_str.is_empty() {
        bail!("old_str cannot be empty");
    }

    let count = content.matches(old_str).count();
    if count == 0 {
        bail!("old_str not found in file: {path_str}");
    }

    let new_content = match occurrence {
        None => {
            // Replace all
            content.replace(old_str, new_str)
        }
        Some(n) => {
            if n == 0 || n > count {
                bail!(
                    "occurrence {n} out of range (file has {count} match(es) of old_str)"
                );
            }
            replace_nth(&content, old_str, new_str, n)
        }
    };

    std::fs::write(&path, &new_content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    let replaced = match occurrence {
        None => format!("{count} occurrence(s)"),
        Some(n) => format!("occurrence {n} of {count}"),
    };

    Ok(format!("Edited {path_str}: replaced {replaced}"))
}

/// Replace the nth occurrence (1-indexed) of `old` in `s` with `new`.
fn replace_nth(s: &str, old: &str, new: &str, n: usize) -> String {
    let mut result = String::with_capacity(s.len());
    let mut remaining = s;
    let mut count = 0;

    while let Some(pos) = remaining.find(old) {
        count += 1;
        result.push_str(&remaining[..pos]);
        if count == n {
            result.push_str(new);
        } else {
            result.push_str(old);
        }
        remaining = &remaining[pos + old.len()..];
    }
    result.push_str(remaining);
    result
}

use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn run(args: &Value, working_dir: &str) -> Result<String> {
    let pattern = args["pattern"].as_str().unwrap_or("");
    let path_str = args["path"].as_str().unwrap_or(".");
    let use_regex = args["regex"].as_bool().unwrap_or(false);
    let case_sensitive = args["case_sensitive"].as_bool().unwrap_or(true);
    let include_glob = args["include_glob"].as_str();

    let base = Path::new(working_dir);
    let target = base.join(path_str);

    let regex = if use_regex {
        let pat = if case_sensitive {
            pattern.to_string()
        } else {
            format!("(?i){pattern}")
        };
        Regex::new(&pat)?
    } else {
        let escaped = regex::escape(pattern);
        let pat = if case_sensitive {
            escaped
        } else {
            format!("(?i){escaped}")
        };
        Regex::new(&pat)?
    };

    let mut results: Vec<String> = Vec::new();
    let mut total_matches = 0usize;
    const MAX_MATCHES: usize = 500;

    search_path(&target, &regex, include_glob, &mut results, &mut total_matches, MAX_MATCHES)?;

    if results.is_empty() {
        Ok("No matches found.".to_string())
    } else {
        if total_matches > MAX_MATCHES {
            results.push(format!("\n... (truncated, showed {MAX_MATCHES} of {total_matches} matches)"));
        }
        Ok(results.join("\n"))
    }
}

fn search_path(
    path: &Path,
    regex: &Regex,
    include_glob: Option<&str>,
    results: &mut Vec<String>,
    total: &mut usize,
    max: usize,
) -> Result<()> {
    if path.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path)?.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let p = entry.path();
            // skip hidden dirs like .git
            if p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }
            search_path(&p, regex, include_glob, results, total, max)?;
        }
    } else if path.is_file() {
        // check include_glob filter
        if let Some(glob_pat) = include_glob {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let pattern = glob::Pattern::new(glob_pat).unwrap_or_else(|_| glob::Pattern::new("*").unwrap());
            if !pattern.matches(file_name) {
                return Ok(());
            }
        }
        search_file(path, regex, results, total, max)?;
    }
    Ok(())
}

fn search_file(
    path: &Path,
    regex: &Regex,
    results: &mut Vec<String>,
    total: &mut usize,
    max: usize,
) -> Result<()> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(()), // skip binary files
    };
    let path_str = path.display().to_string();
    for (i, line) in content.lines().enumerate() {
        if regex.is_match(line) {
            *total += 1;
            if results.len() < max {
                results.push(format!("{}:{}: {}", path_str, i + 1, line));
            }
        }
    }
    Ok(())
}

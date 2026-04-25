use anyhow::Result;
use serde_json::Value;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use crate::config::Config;

pub async fn run(args: &Value, config: &Config) -> Result<String> {
    let cmd = args["cmd"].as_str().unwrap_or("");
    let working_dir = args["working_dir"]
        .as_str()
        .or(config.working_dir.as_deref())
        .unwrap_or(".");

    let requested_timeout = args["timeout_secs"]
        .as_u64()
        .unwrap_or(config.command.default_timeout_secs);

    let timeout_secs = requested_timeout.min(config.command.max_timeout_secs);

    if cmd.is_empty() {
        return Ok("Error: cmd is empty".to_string());
    }

    let child = Command::new("/bin/sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(working_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let result = timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);

            let mut out = format!("Exit code: {exit_code}");
            if !stdout.is_empty() {
                out.push_str("\n--- stdout ---\n");
                out.push_str(stdout.trim_end());
            }
            if !stderr.is_empty() {
                out.push_str("\n--- stderr ---\n");
                out.push_str(stderr.trim_end());
            }
            if stdout.is_empty() && stderr.is_empty() {
                out.push_str("\n(no output)");
            }
            Ok(out)
        }
        Ok(Err(e)) => Ok(format!("Command failed to run: {e}")),
        Err(_) => Ok(format!("Command timed out after {timeout_secs}s: {cmd}")),
    }
}

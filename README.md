# Kazahane

Lightweight AI coding agent powered by OpenAI-compatible APIs.

## Build

```bash
cargo build --release
# Binary: target/release/kazae
```

Requires Rust 1.80+.

## Usage

```bash
kazae "refactor error handling in src/main.rs"
kazae -c myconfig.json "add unit tests"
kazae -m gpt-4o-mini "write a hello world in Go"
kazae -d /path/to/project "summarize this codebase"
kazae --max-tool-calls 30 "quick task"
```

## Config (`Kazae.json`)

```json
{
  "api_key": "",
  "base_url": "https://api.openai.com/v1",
  "model": "gpt-4o",
  "system_prompt": null,
  "max_tool_calls": 100,
  "command": {
    "default_timeout_secs": 30,
    "max_timeout_secs": 300
  },
  "working_dir": "."
}
```

### API Key (priority order)

1. `KAZAHANE_API_KEY` env var
2. `OPENAI_API_KEY` env var
3. `api_key` in config file

### OpenAI-compatible endpoints

```json
{
  "base_url": "https://api.deepseek.com/v1",
  "model": "deepseek-v4-flash"
}
```

## Tools

| Tool      | Description |
|-----------|-------------|
| `grep`    | Search file contents, recursive, regex or literal |
| `glob`    | Find files by glob pattern |
| `read`    | Read file with optional line range |
| `write`   | Create new file (fails if exists) |
| `edit`    | Find-and-replace in file, target specific occurrence |
| `append`  | Append content to file |
| `command` | Run shell command, AI-settable timeout |
| `todo`    | In-memory todo list: add, list, done, delete, edit |

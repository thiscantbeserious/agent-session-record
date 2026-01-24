# Project Overview

## Context

- Records sessions to `~/recorded_agent_sessions/<agent>/`
- Uses asciicast v3 format with native marker support

## Source Code

Explore `src/` for domain modules (config, storage, recording, etc.).

## Auto-Analysis

When `auto_analyze = true` in config, AGR spawns an AI agent after recording to analyze and add markers.

```toml
[recording]
auto_analyze = true
analysis_agent = "claude"
```

See `src/analyzer.rs` for supported analysis agents.

## References

- asciicast v3 spec: https://docs.asciinema.org/manual/asciicast/v3/
- Rust impl: https://github.com/asciinema/asciinema/blob/develop/src/asciicast/v3.rs

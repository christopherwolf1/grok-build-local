# Memory - Local-Only Fork of Grok Build

## Key Learnings

### Architecture Overview
This is a **local-first fork** of Grok Build that modifies the original codebase to work with local LLM runtimes (Ollama, llama.cpp, LM Studio, etc.) without requiring xAI cloud authentication.

### Model System
- **Baked default**: `local` model entry in `default_models.json` with `base_url = "http://127.0.0.1:11434/v1"` (Ollama default)
- **Runtime detection**: Probes well-known ports (Ollama:11434, llama.cpp:8080, LM Studio:1234, etc.)
- **Context window**: Auto-detectable from Ollama's `/api/show` endpoint
- **Model providers**: `[model_providers.<id>]` sections support `kind` presets (ollama, vllm, lmstudio, etc.)
- **API backends**: Default `chat_completions` for local compatibility

### Authentication
- **Credential resolution** (per AGENTS.md):
  1. Per-model `api_key`
  2. Per-model/per-runtime `env_key`
  3. Runtime-level key
  4. Empty key for unauthenticated local servers
- **AuthMode**: `ApiKey`, `WebLogin`, `Oidc`, `External` - keep `ApiKey` for local

### Current State
All phases P00-P13 are marked as "done" in phases/README.md. The fork is designed for seamless local model usage.

## Core Decisions for Local-Only Mode

### What to Remove
1. **xai-grok-update** - Remove auto-update from xAI servers
2. **xai-grok-telemetry** - Remove Mixpanel/OTEL telemetry to xAI
3. **OIDC/device code flows** - Remove browser auth, keep ApiKey
4. **managed_config** - Remove `grok setup` that requires xAI SSO
5. **Remote model fetching** - Remove fallback to xAI's `/v1/models`
6. **Relay sync** - Remove session writeback to xAI backend
7. **Session writeback** - Remove `remote/sync.rs`

### What to Keep
1. **Local model config** - `[model.*]` sections in config.toml
2. **Runtime detection** - Ollama/llama.cpp/LM Studio probing
3. **Model provider kinds** - `kind = "ollama"` presets
4. **ApiKey auth** - `api_key` and `env_key` support
5. **TUI** - Full pager UI with model picker
6. **Tool system** - read/edit/grep/bash/etc.
7. **Workspace** - Local worktree/sandbox support

## Latest Context

### User Requirement
Make this a pure local-only tool:
- No cloud dependencies
- No login required
- Model picker just works
- No environment settings needed
- Avoid collision with official Grok Build TUI

### Implementation Plan
Convert to pure local-only by:
1. Removing update/telemetry code paths
2. Stripping OIDC/bearer auth flows
3. Keeping only baked config + user config for models
4. Removing relay/workspace commands that require xAI backend
5. Simplifying auth to ApiKey-only

### Files to Modify
- `crates/codegen/xai-grok-pager-bin/src/main.rs` - Remove update, telemetry, setup commands
- `crates/codegen/xai-grok-shell/src/auth/*` - Remove OIDC/device code
- `crates/codegen/xai-grok-shell/src/agent/models/fetch.rs` - Remove remote fetch
- `crates/codegen/xai-grok-shell/src/relay/*` - Remove session sync
- `crates/codegen/xai-grok-update/` - Remove crate entirely
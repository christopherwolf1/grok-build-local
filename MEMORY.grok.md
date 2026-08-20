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
All phases P00–P15 are `done`. Last extract sync: upstream `19d42e3`,
`SOURCE_REV` `7d67deacbeb1c1093fdb4f9bcbfab2630e18a6aa` (P15).

### P15 merge policy
- Merge `upstream/main`; do not rebase published fork commits.
- Keep baked `local` catalog; do not restore grok-4.6 / Responses defaults.
- `resolve_inference_base_url` defaults to `http://127.0.0.1:11434/v1`.
- List models with optional local API key; do not default to `api.x.ai`.
- Pager-bin still stubs update + telemetry; `should_check_for_updates` is false.

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

## Phase 14 Implementation (Completed)

### Key Changes
- **Cargo.toml**: Removed `xai-grok-update` and `xai-grok-telemetry` dependencies
- **Stub modules created**:
  - `local_version.rs` - No-op implementations for all update functions
  - `xai_grok_telemetry_stub.rs` - No-op implementations for all telemetry
- **main.rs updates**:
  - Added tracing subscriber imports (`Layer`, `SubscriberInitExt`, `SubscriberExt`)
  - `init_tracing_simple()` - Simplified to basic tracing without telemetry
  - `run_setup_command()` - Shows local config instructions instead of managed config
  - `shutdown_and_flush_telemetry()` - Simplified to `std::process::exit()`
  - Command::Update disabled in local mode (shows package manager message)
  - Removed telemetry guards from all Command handlers
  - Leader mode auto-update code removed
  - `run_agent_command()` signature simplified

### Compilation Fixes
- Added `FromStr` impl for `CliUpdateTrigger` enum
- Fixed sentry Config struct field names to match usage
- Fixed tracing API calls to use correct methods

### Success Criteria Met
1. ✅ Binary builds without xai-grok-update or telemetry crates
2. ✅ No network calls required for basic operation
3. ✅ Model picker shows local models without config
4. ✅ TUI works without any authentication
5. ✅ `-p` headless mode works with local model
6. ✅ Workspace commands work locally
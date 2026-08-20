# Changelog

All notable changes to **grok-build-local** are documented here.

This project versions **itself** independently of upstream Grok Build. The
first product release is **0.1.0** (SemVer `0.x` = still evolving). Cargo
crates may still report upstream `1.0.1`; that is the extract we forked, not
this product’s release number.

Based on SpaceXAI Grok Build public extract `SOURCE_REV`
`5d08d7e4123092567ccd584cd9f99afa2972065c` (crate line 1.0.1).

---

## [0.1.0] — 2026-08-13

First working release of the local-first fork: same TUI and tool surface,
no hosted login in front of a local turn.

### Highlights

- **No `grok login` required.** A reachable OpenAI-compatible server is
  enough to boot and complete a turn.
- **`~/.grok/config.toml` is the catalog.** If any `[model.*]` tables exist,
  those keys **are** `/model`. Official Grok Build uses the same file shape.
- **Chat Completions** is the default local backend (not hosted Responses /
  `grok-4.5`).
- **`/model` Runtimes** — config list first, then models grouped by
  endpoint (`online` / `offline`). `localhost` and `127.0.0.1` on the same
  port are one group, shown as `(127.0.0.1:port)`.
- Hosted Imagine / xAI web search **fail closed** on loopback so a dummy
  local key is never sent to `api.x.ai`. Tools stay in the registry.

### Auth and models

- ACP `local` auth method; session tokens are not sent to loopback or
  non-first-party URLs.
- Credential order: model `api_key` → `env_key` → `LOCAL_API_KEY` →
  optional empty key. `XAI_API_KEY` is not required.
- Baked fallback when config has no `[model.*]`: catalog id `local`,
  `http://127.0.0.1:11434/v1`. `GROK_LOCAL_MODEL` only rewrites that row’s
  **routing slug**.
- `[model_providers.<id>].kind`: `ollama` fills unset URL + Chat
  Completions; explicit fields always win.
- Model list: `GET {base_url}/v1/models` still probes, but discovered ids
  are **not** extra `/model` rows when `[model.*]` is set.
- Unset Ollama `context_window` may be filled in-memory from `POST /api/show`
  (does not write toml). `max_tokens` is not `context_window`.
- Conservative unknown window: 8192 (loopback) — not hosted 200k/500k.
  Set `context_window` on the table when you know it.

### Product surface (unchanged intent)

TUI, headless (`-p`), ACP/stdio, leader, sessions, subagents, skills,
plugins, hooks, MCP, sandbox, plan mode, worktrees, workflows. Full
`grok_build` tool family plus in-tree Codex / OpenCode ports.

### Hosted services

- Auto-update-from-xAI: off / no-op.
- Mixpanel and remote grok.com settings/catalog fetch: off by default.
- Sentry only if explicitly enabled.

### Operator

```sh
cargo run -p xai-grok-pager-bin
# or after `cargo build -p xai-grok-pager-bin --release`:
./target/release/xai-grok-pager
./target/release/xai-grok-pager models    # catalog + port probe; not the TUI
```

Do not use `~/.grok/bin/grok` if you mean this fork.

Example `~/.grok/config.toml`:

```toml
[models]
default = "m5-qwen36-moe"

[model.m5-qwen36-moe]
model = "qwen3.6:35b-a3b"
base_url = "http://127.0.0.1:11434/v1"
name = "M5 Pro Qwen3.6 35B-A3B MoE"
context_window = 131072
```

`[models] default` must be a **catalog key**, not an Ollama slug.

### Known limitations

- Home is still shared with official Grok Build (`~/.grok`). A leftover
  grok.com session can print “logged in with grok.com” without gating
  inference. Dedicated `GROK_HOME` is planned.
- First-token latency on a cold local load can be tens of seconds.
- Runtime display names (Ollama, oMLX, …) use a small port/install map,
  not a name scraped from the server.

### Docs in this tree

- [`README.md`](README.md) — fork front door
- [`AGENTS.md`](AGENTS.md) — agent/operator rules
- [`phases/`](phases/) — capability DAG P00–P13 (all `done` for this slice)

[0.1.0]: https://github.com/christopherwolf1/grok-build-local/releases/tag/v0.1.0

# MEMORY.grok.md

Project memory for the **local-first Grok Build fork**. Append dated entries; do not rewrite history in place unless correcting a factual error.

**Project:** `/Users/christopherwolf/Desktop/Workspace/AI/Code/grok-build-local`
**Upstream:** SpaceXAI Grok Build public extract (`SOURCE_REV` = `5d08d7e4123092567ccd584cd9f99afa2972065c`)
**License:** Apache 2.0 (external PRs not accepted upstream; this is a local fork)

---

## Architecture Decisions

### Fork envelope (2026-08-12)

Keep and may extend:

1. **Product surface** — full TUI, headless (`-p`), ACP/stdio, leader attach, sessions, subagents, skills, plugins, hooks, MCP, sandbox, plan mode, memory, worktrees, workflows. `xai-grok-pager-minimal` is a thinner TUI surface to extend, not a replacement for the full pager.
2. **Tool library** — first-party `grok_build` **and** in-tree Codex / OpenCode ports. Do not delete a tool family to slim the tree. Image/video may error locally; return a clear runtime-missing error instead of removing the tool.

Surgically revise only:

3. **Auth + models** — intended solely for **local models** and **multiple local/LAN runtimes** (Ollama, llama.cpp/llama-server, vLLM, LM Studio, LocalAI, MLX, OpenAI-compatible gateways, optional Anthropic-compatible local proxies). No required `grok login`, no hosted Grok catalog as the default path, no required `XAI_API_KEY`. Cloud BYOK is not a goal.

**Tags:** fork, local-models, product-keep, tools-keep

---

### Runtime / auth target state (2026-08-12)

- First launch must work if a local runtime is reachable.
- First-class multi-runtime config: id, display name, base URL, `api_backend` (`chat_completions` | `responses` | `messages`), optional dummy/env key, extra headers, probe/list models, context-window hints.
- Discover models via `GET /v1/models` (or backend equivalent); merge with explicit `[model.<name>]`.
- Reuse existing `[model.*]` and `[model_providers.<id>]` shape — adapt, do not invent a parallel config language.
- Credential order for this fork: per-model `api_key` → per-model/runtime `env_key` → runtime-level key → optional `LOCAL_API_KEY` / empty key. **Do not** fall back to grok.com session tokens.
- Default `api_backend` for local should be `chat_completions`, not hosted `responses` / `grok-4.5`.
- Per-runtime `stream_tool_calls` and timeouts matter; cold local models can take tens of seconds to first token.
- Unknown context windows must not silently inherit 200k/500k — operator sets them.

**Tags:** auth, models, runtimes, config

---

### Seams to edit first (do not rewrite the harness)

| Seam | Why |
|---|---|
| `crates/codegen/xai-grok-models/default_models.json` | Baked hosted Grok catalog (`grok-4.5`, Responses) |
| `crates/codegen/xai-grok-shell/src/agent/models/` | Fetch / resolve / cache model lists |
| `crates/codegen/xai-grok-shell/src/agent/model_providers.rs` | Provider tables |
| `crates/codegen/xai-grok-shell/src/auth/` | Browser / OIDC / device-code vs local |
| `crates/codegen/xai-grok-auth` | Shared auth types — retarget, do not delete day one |
| `crates/codegen/xai-grok-sampler` | Where inference HTTP actually goes |
| `crates/codegen/xai-grok-pager-bin/src/main.rs` | Auto-update, login-on-boot |
| `crates/codegen/xai-grok-update` | Disable or no-op |
| Pager welcome / `/login` / `/models` | UX must describe runtimes, not grok.com |

Telemetry / Mixpanel / remote sync: gate or stub at call sites if they create dependency holes; do not rip crates if that breaks the graph.

**Tags:** seams, surgical-change

---

## Crate map (orientation)

| Path | Role |
|---|---|
| `xai-grok-pager-bin` | Composition root / `main` |
| `xai-grok-pager` | Full TUI (Elm: Action → dispatch → Effect); ~500 `.rs` + ~250 tests |
| `xai-grok-pager-minimal` | Thinner TUI surfaces |
| `xai-grok-pager-render` | Rendering |
| `xai-grok-shell` | Session host, auth, leader, remote, stdio/headless; ~500 `.rs` |
| `xai-grok-agent` | Agent builder, Markdown+YAML definitions, prompt assembly |
| `xai-grok-tools` | Tool implementations + registry (~240 `.rs`) |
| `xai-grok-sampler` / `xai-grok-sampling-types` | Sampling HTTP / types |
| `xai-grok-models` | Baked `default_models.json` |
| `xai-grok-workspace*` | Host FS, VCS, exec, checkpoints |
| `xai-grok-mcp` | MCP client |
| `crates/common/xai-tool-*` | Tool protocol / runtime / types |
| `third_party/` | Vendored Mermaid — leave unless mermaid is on fire |

Root `Cargo.toml` is **generated** — edit per-crate manifests. ~80 workspace members. Toolchain Rust **1.94.0**, edition **2024**. Need DotSlash + `bin/protoc`.

User guide (product contract): `crates/codegen/xai-grok-pager/docs/user-guide/`.

**Tags:** architecture, crates

---

## Process / conventions

- Target crates: `cargo check -p <crate>` — never full-workspace unless asked.
- Artifact: `xai-grok-pager` (upstream ships as `grok`).
- Comments: short, factual, non-obvious constraints only.
- User-facing errors should name the **runtime** and **URL**, not "xAI API".
- Slash commands and key chords are contracts — update user-guide 03/04 if rebound.
- Headless and ACP must keep working whenever the TUI still works.
- Prefer adapter + config over deleting upstream code.

**Tags:** process, rust, cargo

---

### Phase EDD tracker (2026-08-12)

Capability work lives in [`phases/`](phases/README.md). Each `PXX-*.md` is an
EDD: goal, capabilities, non-goals, seams, acceptance, depends-on / unlocks.

P00–P12 are `done`. **P13** (docs / operator defaults) is the remaining
planned phase. Critical path through P06 is complete. P07 landed as a
thin `kind` preset (Ollama + generic). P10–P12 are integrity audits (no
behavior change).

When a phase status changes, update the EDD, `phases/README.md`, `phases/dag.md`,
`AGENTS.md` phase table, and this file.

**Tags:** phases, edd, dependencies

---

### Graphs: Mermaid, not Graphviz (2026-08-12)

Keep the DAG in Markdown Mermaid.

- [`phases/README.md`](phases/README.md) — one full graph next to the status table.
- [`phases/dag.md`](phases/dag.md) — extra views (critical path, P03∥P04 join, parallel tracks, capability lanes).
- Each `PXX-*.md` — neighborhood only (depends-on + unlocks).

Do **not** add a `.dot` unless we start generating SVGs or a CI check from
front matter. Two render languages would drift.

**Tags:** mermaid, dag, graphviz

---

### Config.toml vs auto-detect (2026-08-12, locked)

**Decision:** `~/.grok/config.toml` is the source of truth and, when any `[model.*]` exists, **is** the `/model` catalog. Auto-detect is a **fill-in layer for unset fields**, not a second product.

This machine already runs **two** runtimes (oMLX `:8000` and Ollama `:11434`). Detect is **per missing field**, never “first healthy port wins the machine.”

Precedence:

```
explicit CLI / config  >  last-used session  >  live probe  >  baked 127.0.0.1:11434
```

Shipped fill-in (2026-08-12):

1. **Silent `context_window` fill** — only if that field is unset on the `[model.*]` table; only for Ollama-shaped URLs (`127.0.0.1|localhost:11434`); `POST /api/show` → `model_info.*.context_length` or `num_ctx`. Does **not** write toml. `GROK_SKIP_RUNTIME_DETECT=1` disables. Skipped in unit tests.
2. **Doctor surface** — `grok models` prints a non-blocking probe of 11434 / 8080 / 1234 / 8000 (`up`/`down`). Does not retarget models or add catalog rows.
3. **Empty config first-run** — still baked `local` + optional `GROK_LOCAL_MODEL`. No write-discovered-models wizard yet.
4. **Architecture** — still deferred. May recommend size/quant later; must not pick Ollama vs MLX vs Unsloth.

Do not: replace config with machine sniff; add `/model` rows from `/v1/models` when `[model.*]` exists; auto-pull weights; hop runtimes mid-session; block boot on probe.

**Tags:** detect, config, P04, P07, doctor

---

### Config.toml vs auto-detect (original note, 2026-08-12)

First write-up; superseded by the locked decision above. Same precedence and layer order. Revisit trigger (real turn + catalog policy) is done.

---

## Immediate Next Steps

**P00–P13 are `done`.** Catalog policy (2026-08-12): if `~/.grok/config.toml`
has any `[model.*]`, those keys **are** `/model`. No baked `local` row and
no discovered `/v1/models` ids as extra selectable rows. Empty config still
gets baked `local` + discovery.
Optional later: write-to-config wizard on empty first-run; hardware
hints (recommend only). Window fill + `grok models` probe shipped.

**Done looks like:** `~/.grok/config.toml` points at localhost; `cargo run -p xai-grok-pager-bin` with no browser; turn completes against a local server.

**Tags:** next, slice-1, P01, P02, P03

---

## Session log

### 2026-08-12 — Orientation + AGENTS.md

- Read repo layout, README, generated workspace, pager/shell/agent/tools docs, `default_models.json`, custom-models guide, model_providers, sampler re-exports. Did **not** read every source line (~thousands of `.rs` files).
- User goal: streamlined fork, but **not** by cutting product or tools — only the cloud model/auth envelope.
- Wrote root [`AGENTS.md`](AGENTS.md) as the working project rule file (north star, crate map, auth seams, tool families, TUI conventions, first-slice done criteria).
- Began this memory file.

**Tags:** session, agents-md, orientation

### 2026-08-12 — Phase EDDs

- Added [`phases/`](phases/README.md): index + mermaid DAG, `_template.md`, P00–P13 EDDs.
- P00 marked `done`. P01–P13 `planned`.
- Updated `AGENTS.md` with phase table, work rules (dependency order, status sync), and first-slice mapping onto P01–P03.

**Tags:** session, phases, edd

### 2026-08-12 — Mermaid DAG views

- Added [`phases/dag.md`](phases/dag.md) (full graph, critical path, join, parallel tracks, capability lanes).
- Neighborhood Mermaid on every EDD + template.
- Decision: Mermaid in Markdown, no Graphviz `.dot` unless we generate diagrams later.

**Tags:** session, mermaid

### 2026-08-12 — P01 identity-free boot landed

- Added ACP auth method `local`; unpinned no-cred users skip `grok login`.
- Version policy no longer exits; auto-update checks disabled.
- Smoke: `--version` and `models` with empty `GROK_HOME` / no `XAI_API_KEY`. Headless `-p` does not ask for login; 401s on hosted `grok-4.5` (P02/P03).
- Tests: `xai-grok-shell` `agent::auth_method` 24 ok; pager `startup_auth_*` 8 ok; `version_policy` 3 ok.

**Tags:** session, P01, local-auth

### 2026-08-12 — P02 runtime/model config landed

- Baked default model is `local` (Chat Completions, 8192 ctx) on `http://127.0.0.1:11434/v1`.
- No `api_base_url` dual-route to `api.x.ai`. Session tokens only on https first-party xAI hosts.
- `LOCAL_API_KEY` then `XAI_API_KEY` then empty.
- `[model.local]` overrides win. `GROK_MODELS_BASE_URL` still skips baked catalog (P04).
- `agent::config` 336 passed. Smoke: `models` lists `local`; headless hits local server (Ollama 404 for slug `local`).

**Tags:** session, P02, local-models

### 2026-08-12 — Config vs auto-detect brainstorm

- Agreed: stay on the config.toml path for now; auto-detect later as fill-in only.
- Sequence: model list (P04) → runtime port probe (P07) → hardware hints last.
- Captured under Architecture Decisions for a later revisit.

**Tags:** session, detect, revisit

### 2026-08-12 — P03 inference landed

- `GROK_LOCAL_MODEL` overrides baked slug; `[model.local] model` wins.
- Sampler idle timeout copied from model config (600s baked).
- 404/connect ACP errors name `[model.local]`, `GROK_LOCAL_MODEL`, `127.0.0.1:11434`.
- Smoke: `GROK_LOCAL_MODEL=llama3.2:latest` headless `-p Hi` → `end_turn`, 2 model calls, no xAI key.

**Tags:** session, P03, ollama

### 2026-08-12 — P04 model discovery landed

- Probe `GET {inference}/models` without login; optional bearer only.
- Catalog is a union: `[model.*]` and baked `local` win on id collision; discovered slugs are added.
- Smoke: `grok models` lists `local` plus this machine’s Ollama tags (`llama3.2:latest`, etc.).

**Tags:** session, P04, discovery

### 2026-08-12 — P05 + P06 local UX / process modes

- `/login` is local-runtime help; welcome fallback is "local runtime"; `grok models` says "Using local runtime (no hosted login)."
- Headless auth-gap copy and pager-minimal header retargeted. Workspace/leader start no longer opens `grok login`.
- ACP eager auth picks `local`.

**Tags:** session, P05, P06

### 2026-08-12 — P08 hosted stubs

- `remote_fetch` defaults false (no grok.com settings/catalog). Local model list still probes.
- Mixpanel off by default; empty-token track is a no-op. Sentry off unless `GROK_ERROR_REPORTING`.
- Relay already opt-in; auto-update already off (P01).

**Tags:** session, P08, offline

### 2026-08-12 — P09 tool-family integrity

- Image/video/xAI web_search do not use a local dummy key against `api.x.ai`.
- Missing Imagine/search backend returns `HOSTED_CAPABILITY_UNAVAILABLE`; tools stay in the registry. `web_fetch` unchanged. Codex/OpenCode ports untouched.

**Tags:** session, P09, tools

---

### 2026-08-12 — P07 thin adapters + P10–P13

- P07: `[model_providers.<id>].kind` with Ollama URL/backend presets;
  generic/openai-compat no-ops; unknown kind warns. Tests in
  `model_providers.rs` (24 passed).
- P10–P12: integrity audits. Sandbox has no login gate; worktrees
  remain; marketplace auto-register default-off and fail-soft;
  session persist has no `auth.json`; compaction uses
  `sampling_config.context_window`.
- P13: README fork banner; user-guide 02/11 + guide README callouts;
  example `config.toml` in `AGENTS.md`.
- Phase table: all P00–P13 `done`.

**Tags:** session, P07, P10, P11, P12, P13

### 2026-08-12 — Detect fill-in + grok models probe

- Locked detect policy in this file: config owns `/model`; dual-runtime machines must not pick a winner.
- In-memory Ollama `/api/show` fill for unset `context_window` only.
- `grok models` prints loopback port reachability. `GROK_SKIP_RUNTIME_DETECT=1` off switch.

**Tags:** session, detect, doctor

---

**Updated:** 2026-08-12
**Project:** `/Users/christopherwolf/Desktop/Workspace/AI/Code/grok-build-local`
**Model:** grok-4.6
**Session:** N/A

# AGENTS.md — Local-models fork of Grok Build

This tree is a **local-first fork** of SpaceXAI's Grok Build (`grok` / `xai-grok-pager`).
Upstream is a periodic extract of an internal monorepo (`SOURCE_REV` at the
repo root). We keep the **product surface** and the **tooling**, and we
**surgically replace cloud identity + hosted-model defaults** with a runtime
layer meant for **local models and multiple inference servers**.

If a change is not required by that sentence, do not make it.

Capability work is tracked as phase EDDs in [`phases/`](phases/README.md).
Read the index before starting implementation. Implement in dependency
order; do not open a phase whose blockers are still `planned` unless that
EDD says it can run in parallel. When a phase starts, ships, or blocks,
update that EDD's status, the table in `phases/README.md`, extra views in
[`phases/dag.md`](phases/dag.md), and `MEMORY.grok.md`. New phases copy
`phases/_template.md` (include a neighborhood Mermaid). Prefer Mermaid in
these Markdown files; do not add a Graphviz `.dot` unless we generate
checked-in diagrams.

---

## North star

1. **Keep (and may extend)** interactive TUI, headless (`-p`), ACP / stdio
   agent mode, leader attach, sessions, subagents, skills, plugins, hooks,
   MCP, sandbox, plan mode, memory, worktrees, workflows.
2. **Keep (and may extend)** the tool library — first-party `grok_build`
   tools **and** the in-tree Codex / OpenCode ports. Do not delete a tool
   family "to slim the tree" unless the user asked.
3. **Revise only the model + auth envelope** so the binary never depends on
   `accounts.x.ai`, `grok.com` login, hosted Grok catalog, Mixpanel, remote
   conversation sync, or auto-update-from-xAI as a required path.

Default posture: this agent talks to **something on the user's machine or
LAN** (Ollama, llama.cpp / llama-server, vLLM, LM Studio, LocalAI, MLX,
OpenAI-compatible gateways, optionally Anthropic-compatible local proxies).
Cloud BYOK is not a goal. Optional dummy/local API keys for servers that
require a header are fine; browser OIDC against xAI is not.

---

## How to work in this repo

- **Read before writing.** The root `Cargo.toml` is generated — treat it as
  read-only. Edit the crate's own `Cargo.toml`.
- **Target crates.** Never `cargo check --workspace` unless the user asked.
  Prefer `cargo check -p <crate>` / `cargo test -p <crate>`.
- **Surgical diffs.** Match surrounding style. No drive-by refactors, no
  reformatting unrelated files, no new abstraction layers for one call site.
- **Do not strip product features** to make the auth change easier. Feature
  flags or no-op stubs are acceptable when a hosted service has no local
  equivalent (image/video gen, announcements, paid usage dashboard).
- **Verify what you touch.** Per-crate tests first. If you change TUI
  behavior, say what you could not exercise in a real terminal.

Toolchain: Rust **1.94.0** (`rust-toolchain.toml`), edition **2024**.
Need **DotSlash** on `PATH` so `bin/protoc` works before proto crates build.

```sh
cargo check -p xai-grok-pager-bin
cargo test  -p xai-grok-config
cargo run   -p xai-grok-pager-bin
```

The artifact is `target/.../xai-grok-pager` (upstream ships it as `grok`).

---

## Crate map (keep this in your head)

| Path | Role |
|---|---|
| `crates/codegen/xai-grok-pager-bin` | Composition root / `main` |
| `crates/codegen/xai-grok-pager` | Full TUI (Elm: Action → dispatch → Effect) |
| `crates/codegen/xai-grok-pager-minimal` | Thinner TUI surfaces — extend, don't replace the full pager |
| `crates/codegen/xai-grok-pager-render` | Rendering |
| `crates/codegen/xai-grok-shell` | Session host, auth, leader, remote, extensions, stdio/headless |
| `crates/codegen/xai-grok-agent` | `Agent` builder, definitions, system prompt assembly |
| `crates/codegen/xai-grok-tools` | Tool implementations + registry |
| `crates/codegen/xai-grok-sampler` | HTTP sampling / streaming / retries |
| `crates/codegen/xai-grok-sampling-types` | Shared sampling types |
| `crates/codegen/xai-grok-models` | `default_models.json` baked defaults |
| `crates/codegen/xai-grok-auth` | Credential types (to be local-runtime oriented) |
| `crates/codegen/xai-grok-config` + `-types` | `~/.grok/config.toml` |
| `crates/codegen/xai-grok-workspace*` | Host FS, VCS, exec, checkpoints |
| `crates/codegen/xai-grok-mcp` | MCP client |
| `crates/common/xai-tool-*` | Tool protocol / runtime / types |
| `third_party/` | Vendored Mermaid stack — leave unless mermaid is on fire |

User guide (product contract):
`crates/codegen/xai-grok-pager/docs/user-guide/`.

---

## Auth + models: the surgical revision

This is the **only** area we intentionally diverge from upstream in a big way.

### Desired end state

- **No required login.** First launch must work if a local runtime is
  reachable. `grok login` against xAI must not be the happy path.
- **Runtimes, not a hosted catalog.** First-class config for
  multiple local servers. Each runtime has: id, display name, base URL,
  API backend (`chat_completions` | `responses` | `messages`), optional
  dummy/env key, extra headers, probe/list models, context-window hints.
- **Discover models from the runtime** (`GET /v1/models` or the backend's
  equivalent) and merge with explicit `[model.<name>]` entries.
- **Defaults live in config + `default_models.json`**, not in remote
  settings from grok.com. Change baked defaults away from `grok-4.5` /
  `api_backend = "responses"` unless a local runtime actually speaks
  Responses.
- **Credential resolution** for this fork:
  1. Per-model `api_key`
  2. Per-model / per-runtime `env_key`
  3. Runtime-level key
  4. Optional `LOCAL_API_KEY` / empty key if the server does not auth
  - **Do not** fall back to `grok login` session tokens or require
    `XAI_API_KEY` for the default path.
- **Keep** the existing `[model.<name>]` and `[model_providers.<id>]`
  shape where it already works (custom models, `api_backend`,
  `extra_headers`, `query_params`). Adapt; don't invent a parallel config
  language.

### Seams to edit (start here, not with a rewrite)

| Seam | Why |
|---|---|
| `xai-grok-models/default_models.json` | Baked hosted Grok catalog |
| `xai-grok-shell/src/agent/models/` | Fetch / resolve / cache model lists |
| `xai-grok-shell/src/agent/model_providers.rs` | Provider tables |
| `xai-grok-shell/src/auth/` | Browser/OIDC/device-code vs local |
| `xai-grok-auth` | Shared auth types |
| `xai-grok-sampler` | Where requests actually go |
| `xai-grok-pager-bin/src/main.rs` | Auto-update, login-on-boot |
| `xai-grok-update` | Disable or no-op for this fork |
| pager welcome / `/login` / `/models` | UX must describe runtimes, not grok.com |

### Do not do

- Rip out `xai-grok-auth` wholesale on day one — retarget it.
- Delete Mixpanel / telemetry crates if that creates a dependency hole;
  gate or stub at the call site.
- Change tool calling, ACP, or session persistence to "simplify models".
- Assume every local server is OpenAI Chat Completions. llama.cpp, vLLM,
  and some MLX builds differ on tools, streaming, and `stream_tool_calls`.
  Per-runtime `api_backend` + `stream_tool_calls` overrides stay.

### Runtime adapter checklist (when adding a server)

1. Probe: can we list models? If not, require explicit `[model.*]`.
2. Which `api_backend`? Default `chat_completions` for local.
3. Tool-call shape (OpenAI tools vs text-only). Disable tools cleanly
   if the runtime cannot emit them.
4. Context window: do not silently use 200k / 500k if unknown — make
   the operator set it.
5. Idle / retry timeouts that make sense for a cold-loaded local model
   (first token can be tens of seconds).
6. One integration test or a mocked `/v1/chat/completions` fixture.

`~/.grok/config.toml` is the same file as official Grok Build. If it
has any `[model.*]` tables, **those keys are the `/model` catalog**.
Baked `local` and `GET /v1/models` ids are not extra rows. Pick a model
with `/model` / `/m`. Set `[models] default` to a **catalog key**
(e.g. `m5-qwen36-moe`), not a runtime slug.

```toml
[models]
default = "m5-qwen36-moe"

[model.m5-qwen36-moe]
model = "qwen3.6:35b-a3b"
base_url = "http://127.0.0.1:11434/v1"
name = "M5 Pro Qwen3.6 35B-A3B MoE"
context_window = 200000
```

`GROK_LOCAL_MODEL` only rewrites the baked `local` slug when there is
**no** `[model.*]` table (empty config / first-run).

Detect is fill-in only: unset `context_window` on an Ollama URL may be
read from `/api/show`. `grok models` probes well-known ports and does
not change the catalog. `GROK_SKIP_RUNTIME_DETECT=1` disables probes.

---

## Tools (keep all families)

Under `crates/codegen/xai-grok-tools/src/implementations/`:

- **`grok_build/`** — first-party (bash, read/edit, grep, list_dir, web,
  tasks, scheduler, plan mode, LSP, workflow, image/video, …).
- **`codex/`** — apply_patch / grep / list_dir / read_file ports.
- **`opencode/`** — bash / edit / glob / grep / read / write / skill /
  todowrite ports.

Image/video tools may fail against a local-only install. That is OK:
return a clear "runtime does not provide this" error. Do not delete the
tool.

Agent definitions: Markdown + YAML frontmatter in `.grok/agents/` or
`~/.grok/agents/`. See `crates/codegen/xai-grok-agent/README.md`.
`promptMode: extend` is the default.

---

## TUI / shell conventions

- Pager is Elm-style: `AppView` owns sessions; `AgentView` is per session;
  input becomes `Action`, dispatch produces `Effect`.
- Prefer extending `xai-grok-pager` over growing `pager-bin`.
- Slash commands and keyboard chords are user-facing contracts. Don't
  rebind without updating
  `crates/codegen/xai-grok-pager/docs/user-guide/03-keyboard-shortcuts.md`
  and `04-slash-commands.md`.
- Headless and ACP must keep working whenever the TUI still works.

---

## Code style

- Rust 2024. `rustfmt.toml` / `clippy.toml` at repo root.
- Comments: short, factual, only for non-obvious constraints. No
  changelog comments. No leftover placeholders.
- Errors: `thiserror` / `anyhow` as the crate already does. User-facing
  strings should name the **runtime** and the **URL**, not "xAI API".
- Tests live next to the crate (`src/` unit tests or `tests/`). Name
  fixtures after the runtime they fake (`ollama_list_models.json`, not
  `foo.json`).
- Do not add workspace members unless a new crate is the cleanest seam
  (e.g. `xai-grok-runtime` if provider probing outgrows `models/`).

---

## Phases

Index and DAG: [`phases/README.md`](phases/README.md).

| ID | Capability | Status | Depends on |
|---|---|---|---|
| P00 | Program contract (this file, memory, EDDs) | done | — |
| P01 | Identity-free boot | done | P00 |
| P02 | Runtime and model config | done | P01 |
| P03 | Inference and sampling | done | P02 |
| P04 | Model discovery and catalog | done | P02 |
| P05 | TUI local UX | done | P01, P03, P04 |
| P06 | Headless, ACP, leader | done | P03, P04 |
| P07 | Multi-runtime adapters | done | P03, P04 |
| P08 | Hosted-service stubs | done | P01 |
| P09 | Tool-family integrity (keep/extend) | done | P03 |
| P10 | Workspace, sandbox, worktrees | done | P01 |
| P11 | Extensibility (MCP, skills, plugins, hooks) | done | P01 |
| P12 | Sessions, memory, subagents, plan mode | done | P03 |
| P13 | Docs and operator defaults | done | P05, P06, P07 |
| P14 | Local-only ablation | done | P02 |
| P15 | Upstream extract sync | done | P14 |

**Critical path:** P01 → P02 → P03 → P05 / P06.  
**Parallel after P01:** P08, P10, P11. **After P02:** P03 ∥ P04.

## What "done" looks like for the first fork slice

That slice is **P01 + P02 + enough of P03** that a developer can:

1. Point `~/.grok/config.toml` at `http://127.0.0.1:11434/v1` (or another
   local server).
2. Run `cargo run -p xai-grok-pager-bin` with **no browser login**.
3. List models from that runtime and start a TUI or `grok -p` turn.
4. Use read / edit / bash / grep as today.
5. Attach via ACP/stdio the same way upstream documents.

Everything else (MCP, plugins, mermaid, sandbox, …) should still compile
and behave as upstream unless it hard-requires grok.com. Those surfaces
have their own later phases (P08–P12); do not gut them while landing
the first slice.

---

## When you are unsure

Prefer **adapter + config** over **deleting upstream code**. Leave a
short comment at the seam naming the local-runtime invariant. Ask the
user before removing a product feature or a tool family.
}

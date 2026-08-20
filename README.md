# Grok Build local

Working name for this checkout: **grok-build-local**. Rename later if
a better product name sticks; the binary is still `xai-grok-pager`.

Customized version of the Grok Build TUI harness focused on local
models and routers first.

A **local-first fork** of [SpaceXAI Grok Build](https://x.ai/cli): the same
full-screen TUI, headless/`-p` mode, ACP/stdio, tools, skills, sessions, and
worktrees — aimed at **Ollama, oMLX, llama.cpp, vLLM, LM Studio, Unsloth**, or any
OpenAI-compatible LAN server.

**No `grok login`.** Hosted weekly limits and grok.com sessions do not sit in
front of a local turn, even for your local models and routers. This also adopts your existing `~/.grok/config.toml` as the model catalog (same file
shape as official Grok Build).

Upstream product docs remain at [x.ai/cli](https://x.ai/cli). This tree is not
the official installer and does not accept external PRs the way the public
extract describes.

`SOURCE_REV` records the monorepo SHA this fork started from
(`7d67deacbeb1c1093fdb4f9bcbfab2630e18a6aa`).

---

## What you are used to vs. what has changed

| Kept | Changed |
|------|--------|
| TUI, headless, ACP, leader, sandbox, plan mode, memory, worktrees, workflows | No required hosted login or hosted catalog |
| Full tool library (`grok_build` + in-tree Codex/OpenCode ports) | Default backend is Chat Completions, not hosted Responses |
| Skills, plugins, hooks, MCP | Mixpanel / remote grok.com fetch / auto-update-from-xAI off by default |
| `[model.*]` / `[model_providers.*]` | `/model` lists **your** tables; Runtimes section groups by endpoint |

Imagine / xAI web search stay in the registry but **fail closed** on loopback
(`HOSTED_CAPABILITY_UNAVAILABLE`) so a dummy local key is never sent to
`api.x.ai`. In future updates, these will probably be completely removed.

Working rules for agents: [`AGENTS.md`](AGENTS.md). Capability DAG:
[`phases/`](phases/). 
---

## Quick start

Requirements: **Rust** (see [`rust-toolchain.toml`](rust-toolchain.toml)),
**[DotSlash](https://dotslash-cli.com)** on `PATH` (for [`bin/protoc`](bin/protoc)),
and a local OpenAI-compatible server. macOS and Linux are the supported
build hosts.

```sh
cargo install dotslash          # once
ollama pull llama3.2            # or use models you already have
cd grok-build-local
cargo run -p xai-grok-pager-bin
```

That launches the TUI (`target/debug/xai-grok-pager`). Do **not** use
`~/.grok/bin/grok` if you mean this fork — that is usually the official
binary.

Headless:

```sh
cargo run -p xai-grok-pager-bin -- -p "Reply with pong only." --max-turns 1
```

List the catalog and probe loopback ports (does **not** open the TUI):

```sh
cargo run -p xai-grok-pager-bin -- models
```

Release binary: `cargo build -p xai-grok-pager-bin --release` →
`target/release/xai-grok-pager`.

---

## Point it at your models

If `~/.grok/config.toml` has any `[model.*]` tables, **those keys are `/model`**.
Discovered Ollama ids and the baked `local` row are not extra entries.

```toml
[models]
default = "m5-qwen36-moe"

[model.m5-qwen36-moe]
model = "qwen3.6:35b-a3b"
base_url = "http://127.0.0.1:11434/v1"
name = "M5 Pro Qwen3.6 35B-A3B MoE"
context_window = 131072

[model_providers.home]
kind = "ollama"   # fills unset URL + chat_completions only
```

- Catalog **key** (`m5-qwen36-moe`) is what `/model` and `[models] default`
  use. The `model =` field is the slug sent to the runtime.
- `localhost` and `127.0.0.1` on the same port are one runtime in the
  **Runtimes** section (`online` / `offline`).
- Empty config still gets baked `local` at `http://127.0.0.1:11434/v1`.
  `GROK_LOCAL_MODEL` then only rewrites that row’s slug.
- Optional `GROK_SKIP_RUNTIME_DETECT=1` turns off port probes and Ollama
  `/api/show` window fill.
- `max_tokens` is not `context_window`. Set the window on the table if you
  do not want the conservative 8K/200K fallback.

User-guide callouts: [authentication](crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md)
(local, no login) and [custom models](crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md).

This home directory is shared with official Grok Build (`~/.grok`). Isolation
via a dedicated `GROK_HOME` is planned, not required.

---

## Building and developing

- **protoc:** DotSlash runs [`bin/protoc`](bin/protoc), or set `$PROTOC`.
- Target **one crate**. Full-workspace `cargo` is too slow.

```sh
cargo check -p xai-grok-pager-bin
cargo test -p xai-grok-shell --lib
cargo clippy -p xai-grok-shell
cargo fmt --all
```

The root `Cargo.toml` is **generated** — treat it as read-only.

| Path | Role |
|------|------|
| `crates/codegen/xai-grok-pager-bin` | `main`; binary name `xai-grok-pager` |
| `crates/codegen/xai-grok-pager` | TUI (Elm-style Action → Effect) |
| `crates/codegen/xai-grok-shell` | Session host, models, ACP, headless |
| `crates/codegen/xai-grok-tools` | Tool implementations |
| `crates/codegen/xai-grok-workspace` | FS, VCS, exec, checkpoints |
| `phases/` | Fork capability EDDs (P00–P13) |

---

## License

First-party code: **Apache 2.0** — [`LICENSE`](LICENSE).

Vendored and ported code stays under its original licenses:
[`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES),
[`crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md`](crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md),
[`third_party/NOTICE`](third_party/NOTICE).

This is a personal/local fork of the public extract. Official Grok Build
does not accept external contributions; neither does this tree unless the
owner says otherwise.

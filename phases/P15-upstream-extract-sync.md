# P15 — Upstream extract sync

| Field | Value |
|---|---|
| Status | done |
| Owner | — |
| Depends on | P14 |
| Unlocks | future extract syncs |
| Primary crates | `xai-grok-pager-bin`, `xai-grok-shell`, `xai-grok-models` |

## Neighborhood

```mermaid
flowchart LR
  P14[P14 local-only ablation] --> P15[P15 upstream extract sync]
```

## Goal

Record a new extract sync point and merge `upstream/main` (`48271133`,
`SOURCE_REV` `be7ce6e8cffe46d20bef9834b211616082ee866b`) while keeping the
local-first envelope: no required login, baked `local` catalog, loopback
inference default, stubbed update/telemetry at the binary. Previous extract
was `37949780` / `c4ea71cfdbcdb21e32e41bc25a0043d7d4836714`.

## Capabilities

- Fork tracks official extract SHA in `SOURCE_REV` after this merge.
- Product crates from the extract compile; TUI/headless/ACP surfaces remain.
- Operator still uses `[model.*]` + local runtimes as `/model`.

## Non-goals

- Rebase of published fork history.
- Restoring grok-4.6 / Responses as baked defaults.
- Relinking live `xai-grok-update` / Mixpanel into the pager binary.
- GROK_HOME isolation.

## Seams

| Path | Change |
|---|---|
| `SOURCE_REV` | `be7ce6e8cffe46d20bef9834b211616082ee866b` |
| `default_models.json` | Keep fork `local` catalog |
| `pager-bin/src/main.rs` | Upstream config loader + `memory_enabled_override`; keep stubs |
| `shell/.../agent/config.rs` | Upstream APIs + local URL/window/env slug |
| `shell/.../remote/client.rs` | Optional local API key list; no `api.x.ai` default list |

## Acceptance

- [x] `cargo check -p xai-grok-pager-bin`
- [x] `cargo test -p xai-grok-pager-bin --bins` (39 unit tests; TUI not exercised)
- [x] `cargo test -p xai-grok-shell --lib models_fetch_endpoint_matches_auth_mode`
- [x] `cargo check -p xai-grok-pager-bin` after extract `48271133`
- [x] Baked default is `local`, not `grok-4.6`
- [x] `should_check_for_updates` remains `false`

# P10 — Workspace, sandbox, and worktrees

| Field | Value |
|---|---|
| Status | done |
| Owner | — |
| Depends on | P01 |
| Unlocks | — |
| Primary crates | `xai-grok-workspace`, `xai-grok-workspace-types`, `xai-grok-workspace-client`, `xai-grok-sandbox`, `xai-fast-worktree` |

## Neighborhood

```mermaid
flowchart LR
  P01[P01 boot] --> P10[P10 workspace sandbox worktrees]
```

No downstream phase. Full DAG: [dag.md](dag.md).

## Goal

Host filesystem, VCS, execution, checkpoints, sandbox, and worktrees
keep working without hosted identity. Integrity phase — no feature cut.

## Capabilities

- Workspace root, ignore rules, checkpoints behave as upstream.
- Sandbox can still confine commands when the operator enables it.
- Fast worktrees / overlay isolation still available.
- No sandbox policy that requires an xAI-signed config.

## Non-goals

- Do not rewrite the sandbox backends.
- Do not make sandbox mandatory for local models.

## Seams

| Path | Change |
|---|---|
| workspace + sandbox init | Strip identity assumptions if any |
| managed/requirements config | Must not re-require xAI |

## Acceptance

- [x] Agent can edit files in the workspace after P01 boot (no sandbox
      login gate).
- [x] Sandbox smoke (existing examples/tests) still builds.
- [x] Worktree-related slash/commands remain present (`/fork --worktree`,
      `xai-fast-worktree`, pager git_info / disk_usage).

## Risks

- Managed enterprise config could reintroduce hosted URLs — treat as P02
  overlap; note any findings here.

## Notes

Integrity audit 2026-08-12: no `grok login` in sandbox; worktree fork
path present in pager `headless.rs` / `git_info.rs`. No code change
required.

# Phase DAG (Mermaid)

Canonical **multi-view** graphs. The same edges live in [`README.md`](README.md).
Each EDD has a **neighborhood** diagram (incoming + outgoing only). Do not
add a Graphviz `.dot` unless we need generated images or a CI check; Mermaid
is what GitHub, the pager, and most editors already render from this tree.

When you add an edge, update: this file, `README.md`, the two EDD
neighborhoods, `AGENTS.md`, and `MEMORY.grok.md`.

## Full dependency graph

```mermaid
flowchart TD
  P00[P00 program contract]
  P01[P01 identity-free boot]
  P02[P02 runtime and model config]
  P03[P03 inference and sampling]
  P04[P04 model discovery]
  P05[P05 TUI local UX]
  P06[P06 headless ACP leader]
  P07[P07 multi-runtime adapters]
  P08[P08 hosted-service stubs]
  P09[P09 tool-family integrity]
  P10[P10 workspace sandbox]
  P11[P11 extensibility]
  P12[P12 sessions memory subagents]
  P13[P13 docs and defaults]

  P00 --> P01
  P01 --> P02
  P01 --> P05
  P01 --> P08
  P01 --> P10
  P01 --> P11
  P02 --> P03
  P02 --> P04
  P03 --> P05
  P04 --> P05
  P03 --> P06
  P04 --> P06
  P03 --> P07
  P04 --> P07
  P03 --> P09
  P03 --> P12
  P05 --> P13
  P06 --> P13
  P07 --> P13
```

## Critical path

First local turn (TUI or headless) is unblocked when P03 is done; P05/P06
are the product-facing ends of the path.

```mermaid
flowchart LR
  P00 --> P01 --> P02 --> P03
  P03 --> P05
  P03 --> P06
  P05 --> P13
  P06 --> P13
```

P04 is **not** on this path but is required before P05/P06 can list models.
Treat P03 ∥ P04 after P02, then join.

```mermaid
flowchart LR
  P02 --> P03
  P02 --> P04
  P03 --> P05
  P04 --> P05
  P03 --> P06
  P04 --> P06
```

## Parallel tracks after P01

Inference is not blocked by stubs or host/extensibility integrity.

```mermaid
flowchart TD
  P01[P01 identity-free boot]
  P01 --> P02[P02 config]
  P01 --> P08[P08 hosted stubs]
  P01 --> P10[P10 workspace sandbox]
  P01 --> P11[P11 extensibility]
  P02 --> P03[P03 inference]
  P02 --> P04[P04 discovery]
```

## Capability lanes

Same DAG, grouped by what we are protecting vs revising.

```mermaid
flowchart TB
  subgraph revise [Revise: auth and models]
    P01
    P02
    P03
    P04
    P07
    P08
  end
  subgraph product [Keep: product shells]
    P05
    P06
    P13
  end
  subgraph keep [Keep: tools and platform]
    P09
    P10
    P11
    P12
  end
  P00 --> P01
  P01 --> P02
  P01 --> P08
  P01 --> P10
  P01 --> P11
  P02 --> P03
  P02 --> P04
  P03 --> P05
  P04 --> P05
  P01 --> P05
  P03 --> P06
  P04 --> P06
  P03 --> P07
  P04 --> P07
  P03 --> P09
  P03 --> P12
  P05 --> P13
  P06 --> P13
  P07 --> P13
```

## Why not Graphviz

| | Mermaid in Markdown | Graphviz `.dot` |
|---|---|---|
| Renders in GitHub / pager / most previewers | Yes | Needs `dot` or a generated SVG |
| Lives next to the EDD that owns the edges | Yes | Second source of truth |
| Good for 14-node DAGs | Yes | Better for huge generated graphs |
| CI can check structure | Possible later | Easier to parse |

Use Graphviz only if we start **generating** the graph from front matter
and want a checked-in SVG. Until then, one Mermaid DAG, many views.

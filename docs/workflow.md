# Workflow

> **AI Context Summary**: The project uses a planner/builder agent pattern for multi-part features. The planner agent reads specs and designs implementation plans; the builder agent executes them. Both live in `.claude/agents/`. Parallelize independent subproblems using the Task tool; serialize tasks that touch shared files (`main.rs`, `provider.rs`).

## Overview

Complex features are decomposed and executed in parallel using Claude Code's Task tool. Two agents formalize this pattern:

- **`.claude/agents/planner.md`** — Reads specs, analyzes the codebase, produces a step-by-step implementation plan with file paths
- **`.claude/agents/builder.md`** — Receives a plan, makes edits, runs tests, reports completion

## Standard Workflow

```
Feature request
      │
      ▼
Decompose into N independent subproblems
      │
      ├─> Task(planner, subproblem_1) ─┐
      ├─> Task(planner, subproblem_2)  ├─> collect plans in parallel
      └─> Task(planner, subproblem_N) ─┘
                                        │
                              Resolve file-overlap conflicts
                                        │
      ┌─────────────────────────────────┘
      │
      ├─> Task(builder, plan_1) ─┐  (independent plans)
      ├─> Task(builder, plan_2)  ├─> parallelize
      └─> ...                   ─┘
      │
      └─> Task(builder, plan_X)       (plans sharing files — sequential)
```

## When to Use This Pattern

**Use planner/builder** when the feature:
- Touches 3+ files
- Has clearly independent subproblems
- Requires reading multiple specs before implementing

**Implement directly** (no agents) for:
- Single-file changes with clear scope
- Bug fixes with an obvious root cause
- Incremental changes to existing patterns

## Spawning a Planner Agent

```
Task(
  subagent_type: "planner",
  prompt: "Design the implementation for [feature].
           Read specs/architecture.md and specs/dyt-daemon.md first.
           Files likely involved: [list].
           Constraint: audio callback must remain real-time safe.
           Output: numbered step-by-step plan with exact file paths."
)
```

## Spawning a Builder Agent

```
Task(
  subagent_type: "builder",
  prompt: "Implement the following plan:
           [paste plan output from planner]

           Constraints:
           - Audio callback (capture.rs): no locks, no allocations
           - Cross-platform: must compile on Ubuntu and Windows
           - hound WAV: use Cursor::new(&mut buf) + finalize() pattern
           - New STT backends: add to dyt-daemon/src/provider/, register in provider.rs"
)
```

## File Overlap Resolution

Before spawning builders in parallel, check whether plans edit the same file:

| Situation | Action |
|-----------|--------|
| Plans edit different files | Parallelize freely |
| Plans edit the same file | Run sequentially; second builder reads first builder's output |
| Plans touch `main.rs` or `provider.rs` | Sequence — these are shared entry points |

## Key Constraints to Pass to Builders

Always include these in builder prompts when relevant:

- **Audio callback real-time safety**: `capture.rs` callback — no mutex, no allocation, no I/O
- **Cross-platform**: code must compile on Ubuntu and Windows; use `cpal`, `arboard`, `dirs` abstractions
- **hound WAV pattern**: `Cursor::new(&mut buf)` + `finalize()` — see CLAUDE.md Notes
- **Extension point**: new STT backends go in `dyt-daemon/src/provider/` and are registered in `from_config()` in `provider.rs`
- **Config path**: resolved via `dirs` crate; never hardcode `~/.config` or `%APPDATA%`

## Cross-References

- Agent definitions: `.claude/agents/planner.md`, `.claude/agents/builder.md`
- Architecture (read before planning): `docs/architecture.md`
- Specs (authoritative design decisions): `specs/`

---
name: adr
description: Draft an architecture decision record for Linca as docs/decisions/NNNN-title.draft.md. Use when a conformance case and the spec disagree, when a spec rule is ambiguous or missing, or when a change would alter the language, the IR, the plugin protocol or the CLI. Agents write drafts only; the human reviews a draft by renaming it.
argument-hint: <the question to decide>
---

## When an ADR is needed

It is needed when the answer changes what users or plugins observe: language semantics, diagnostics, IR shape,
plugin protocol, `linca.toml`, CLI, exit codes. It is also needed for a compiler architecture choice that
the RFC does not already settle.

It is not needed for internal refactors, naming inside the compiler or test helpers. For those, just do the work.

## Lifecycle

| File | Who | Meaning |
|---|---|---|
| `NNNN-kebab-title.draft.md` | agent or human | proposal, not reviewed yet |
| `NNNN-kebab-title.md` with `Status: accepted` | human only | the rule; the spec follows it |
| `NNNN-kebab-title.md` with `Status: rejected` | human only | considered and turned down; do not propose it again |
| `NNNN-kebab-title.md` with `Status: superseded by ADR-MMMM` | human only | replaced by a later decision |

Renaming a draft is the human's review. Hooks block an agent from writing, editing or renaming anything in
`docs/decisions/` except a file named `NNNN-kebab-title.draft.md`.

## Steps

1. Read the titles in `docs/decisions/`, drafts included.
   - A final record on the same question exists: do not reopen it. Cite it as `ADR-NNNN` and, if you
     think it is wrong, say so in your report. If it is `rejected`, do not propose the same option again.
   - A draft on the same question exists: extend that draft instead of creating a second.
2. Number: one above the highest `NNNN` in the directory, counting drafts. Numbers are never reused, even
   when a draft is deleted.
3. File: `docs/decisions/NNNN-kebab-title.draft.md`. The title is lowercase ASCII words joined by `-`,
   stated as the decision: `0007-nullable-collections-are-not-allowed`.
4. Fill in `template.md` from this skill's folder. Give at least two real options, each with its consequences
   for users, plugins and the compiler. Link spec sections and case ids. Keep the text under about 60 lines;
   diagrams do not count. Leave "Decision" empty; that section is the human's.
5. Add a Mermaid diagram only where the options differ in structure: order of compiler phases, data flow
   between modules, module or crate dependencies, the shape of an IR object, or a sequence between the
   compiler and a plugin. Draw each affected option the same way, so they can be compared side by side.
   Do not draw rules that are a single check or a naming convention.
   - The diagram supplements the text; every consequence is still written out in words.
   - Use only `flowchart`, `sequenceDiagram`, `stateDiagram-v2` or `classDiagram`, with no styling or themes.
     Keep node labels short and plain; quote a label that contains brackets, colons or quotes. 
6. Refer to it everywhere as `ADR-NNNN`, never by file path, because the path changes on review.
7. If the question came up during an impl task, say in your report that the task is blocked on `ADR-NNNN`.

# Linca — guide for agents

Spec-first schema language. The compiler is Rust (Cargo workspace, three crates; see
`docs/Compiler RFC.md` §10). Sources of truth, in this order:

1. `tests/data/lang/` — the conformance suite. Expected diagnostics are the `//~` annotations in the case files (rules in tests/data/lang/README.md).
2. `docs/Specification.md` (language), `docs/IR Specification.md` + `schema/linca-ir.schema.json` (IR).
3. `docs/Compiler RFC.md` (architecture, phases, CLI).
4. `docs/decisions/` — ADRs: why a rule is the way it is. `NNNN-title.md` is reviewed (accepted, rejected,
   superseded); `NNNN-title.draft.md` is an unreviewed proposal. Do not undo an accepted ADR, and do not propose
   a rejected option again. Refer to records as `ADR-NNNN`.

The `linca-spec` skill maps codes, sections, IR keys and phases. Use it instead of reading whole documents.

## Modes

The session mode is printed at start. Hooks enforce it.

- **impl** (default): write code in `crates/`. Spec, schema, cases, examples and this file are locked.
- **spec** (`LINCA_MODE=spec`): write spec, ADRs and cases. `crates/` is locked.

## Hard rules

- A failing case is fixed in the compiler. If you believe the case or the spec is wrong, write an ADR
  draft (`adr` skill, `docs/decisions/NNNN-title.draft.md`) and stop. Do not work around it.
- In `docs/decisions/` you write only `*.draft.md` files. Never rename a draft or edit a reviewed record:
  that is the human's review.
- Never bless or regenerate expected output (IR snapshots, `LINCA_BLESS`). The human reviews diffs.
- No special-casing of case names, file paths or test inputs. No `todo!()` or `unimplemented!()` on a path a case reaches.
- Diagnostic codes come only from the table in Specification §12. Never invent or reuse a code.
- Deterministic output: no iteration over `HashMap`/`HashSet`, sort file lists, stable order everywhere (RFC §3, decision 10).
- A phase never mutates the output of an earlier phase. `lower` (phase 10) contains no decisions (RFC §4, §5.10).
- If code needs a long comment to justify itself, the code is wrong. Fix the code.

## Commands

- `cargo test -p linca-compiler --test lang -- <filter>`: case runner, one test per case id (e.g. `E05`, `valid/08_relations`)
- `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`
- `cargo run -p linca -- check --format json <path>`, `cargo run -p linca -- ir --only <id>`

## Skills

- `/implement-cases <filter>`: implement until the selected cases pass, then the `reviewer` subagent checks the diff.
- `/spec-change <what>` (spec mode): change a rule in the spec together with its cases.
- `adr` (used by the agent on its own, or by you as `/adr <question>`): draft a decision record as `NNNN-title.draft.md`.

## Lessons

Each line below was added after a real mistake. Keep the list short and concrete.

- (empty)

---
name: implement-cases
description: Implement Linca compiler code until a selected set of conformance cases passes, then get an adversarial review of the diff. Use for every compiler task in impl mode.
argument-hint: <case filter, e.g. E01 | E05 | valid/08_relations | milestone 1>
disable-model-invocation: true
---

Task: make the cases selected by `$ARGUMENTS` pass without breaking any other case.

## 1. Scope

- Translate the filter into case ids, which are the paths under tests/data/lang/ (diagnostics/E0502_…/bad, valid/08_relations/…). A milestone means its "done when" line in Compiler RFC §13.
- For each code involved, find its rule with the `linca-spec` skill: spec section, owning phase, module.
- State the scope in two or three lines before writing code: cases, phases, modules.

## 2. Loop

1. Run `cargo test -p linca-compiler --test lang -- <filter>` and read the failures.
2. Read the spec section of the rule, not only the case. Implement the rule as written.
3. Put the code in the module that owns the phase (RFC §10). Add unit tests next to it for edge cases
   the conformance cases do not cover.
4. Repeat until the selected cases pass.

If the runner does not exist yet, building it is the first task (RFC §11). It generates one test per case id
(e.g. with `libtest-mimic`), so a cargo test filter selects cases.

## 3. When a case looks wrong

Stop changing code for that case. Use the `adr` skill to write a draft (NNNN-title.draft.md): the case id, the spec
section, what the case expects, what you think the spec says, and the options. Leave the case failing and
report it.

## 4. Done

All of these hold:

- the selected cases pass;
- `cargo test --workspace` has no new failures;
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` are clean;
- the `reviewer` subagent returns PASS on the diff. If it returns FAIL, fix the blockers and majors and
  run it again (at most three rounds, then report what is left).

## 5. Report

Keep it short: cases now passing, cases still failing and why, ADR drafts created (as ADR-NNNN), reviewer findings you
did not fix, and anything the human should check by hand. Do not commit. The human reviews and commits.

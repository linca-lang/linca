---
name: spec-change
description: Change or add a Linca language rule consistently across the specification, the diagnostic code table, the IR documents and the conformance cases. Only in spec mode (LINCA_MODE=spec).
argument-hint: <the rule to add or change, or an ADR number>
disable-model-invocation: true
---

Change: `$ARGUMENTS`.

If the session mode is not `spec`, stop and say so. Compiler code is out of scope here.

## 1. Decision first

- If the change alters behaviour users or plugins can observe, a reviewed ADR with `Status: accepted` must exist
  in `docs/decisions/` (a file without `.draft`). A draft is not enough.
- If there is none, or only a draft, write or extend the draft with the `adr` skill and stop. The human reviews
  it by renaming the file.
- If an accepted ADR exists, follow it exactly. Name it as `ADR-NNNN` in every section you touch.

## 2. Specification

- Change the section that states the rule. Use MUST/MUST NOT for rules, and name the diagnostic code next to the rule.
- A new error gets a new code in the right range of §12 (01xx lexical ... 08xx services). A code is never
  reused or renumbered. A retired code stays in the table, marked retired.
- Update §15 "Decisions and open questions" if the change settles or opens one.
- If the change touches the IR: update `docs/IR Specification.md`, `schema/linca-ir.schema.json` and
  `examples/shop/shop.ir.json` together, then check that the example validates against the schema.

## 3. Cases

For every new or changed rule:

- `tests/data/lang/diagnostics/<CODE>_<snake_name>/bad.linca`: the smallest file that breaks the rule,
  with `//~ <CODE>` at the end of the primary line.
- `ok.linca` next to it: the smallest change that makes it valid. No annotations, no warnings at all,
  naming conventions followed (W0001).
- Multi-module cases use `bad/` and `ok/` directories as schema roots.
- First two lines of every file:
  ```
  // Linca test case | spec <version> | section <n.m>
  // <what the case shows, one line>
  ```
- If the spec does not fix the primary line, choose one and add it to the table in
  `tests/data/lang/README.md`.
- A valid construct gets a file under `valid/<NN_section>/`.

## 4. Consistency check

- Every code in §12 has at least one `bad` case; every annotated code is in §12.
- Section numbers in case headers exist in the spec.
- Grep the three documents for the old wording of the rule and update every mention.

## 5. Report

The files changed, the codes added or changed, and a commit message draft that names the ADR.

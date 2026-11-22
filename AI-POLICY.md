# AI policy

Linca is developed with the help of AI coding agents (Claude Code strogly recommended at this moment). This document says what an agent may do in
this repository, what stays with a human, and how AI use is disclosed. How the work is organised day to day is in
[docs/AI Workflow.md](<docs/AI Workflow.md>); how the rules are enforced is in [.claude/README.md](.claude/README.md).

## Principles

1. **A human decides what Linca is.** The language, the IR, the plugin protocol and the CLI are defined by the
   specification, the decision records and the conformance cases. A human accepts every one of them.
2. **Tests are written apart from the code that passes them.** Conformance cases are written in a spec session;
   compiler code is written in an impl session, which cannot change the cases.
3. **The human who commits is responsible for the commit**, whoever or whatever wrote its lines.

## What an agent may do

| | Agent | Human only |
|---|---|---|
| Compiler code (`crates/`) | writes it in an impl session | reviews and commits |
| Unit and property tests inside `crates/` | writes them | reviews |
| Specification, IR schema, examples | drafts changes in a spec session | decides, reviews, commits |
| Conformance cases (`tests/data/lang/`) | drafts them in a spec session, from a rule the human gives | decides whether the language should behave so |
| Decision records (`docs/decisions/`) | writes drafts (`NNNN-title.draft.md`) | reviews by renaming; sets accepted or rejected |
| Expected output (IR snapshots) | may generate | accepts (bless) after review |
| Commits | proposes | commits; nothing is pushed by an agent |
| This policy, the workflow, the harness (`.claude/`) | — | edits by hand |

An agent never changes a test, a case or the spec to make its own code pass. When it believes a case or the spec is
wrong, it writes an ADR draft and stops.

## Contributions from others

When the project accepts outside contributions:

- say in the pull request whether and how AI tools were used;
- you must understand and be able to explain every line you submit;
- the rules above apply to your agent as well: no changes to cases, the spec or decisions to make code pass.

## Changing this policy

Only a human edits this file. Changes are committed on their own, with the reason in the commit message.

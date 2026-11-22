# ADR-0004. Naming warning on a name that already has an error

- **Proposed:** 2026-11-22, by TeseySTD
- **Affects:** language | compiler
- **Spec:** Specification §3.2, §3.3, §12 (E0104, W0001), Compiler RFC §5.4, §5.7, §5.12
- **Cases:** tests/data/lang/diagnostics/E0104_keyword_as_declaration_name,
  tests/data/lang/diagnostics/E0104_keyword_as_operation_name
- **Status:** accepted

## Context

`record model { ... }` is E0104 (phase 4, resolve). `model` is lowercase, so it also violates the PascalCase
convention (W0001, phase 7, check). The case files say "No W0001 is expected", and the runner requires the
set of reported codes to equal the annotated set, so a second warning fails the case. The spec has no rule
that says why: §3.2 and §12 are silent, and RFC §5.12 poisons nodes only in general terms.
Affected: users (one root cause, one message) and the compiler (phase 7 sees the declaration or not).

## Options

### A. Report both E0104 and W0001
No special rule. Consequences:

Positive:
- Phases are independent; no knowledge of earlier errors in `check`.

Negative:
- Two diagnostics for one mistake; the warning adds noise ("rename to `Model`") that would lead the user
  to another invalid name.
- The two existing cases fail and must be changed.

### B. Suppress W0001 on a name that has an error
A name rejected with E0104 is poisoned (RFC §5.12); `check` does not warn about it. Consequences:

Positive:
- Matches the existing cases; one diagnostic per root cause.
- Fits the rule that later phases skip poisoned nodes.

Negative:
- Must be stated in the spec: "W0001 is not reported for a name that has an error". The rule needs to
  say whether it extends to other errors (E0701 duplicate name, E0103 recovery) or only E0104.
- The warning appears after the user fixes the error, so a fix can reveal a new diagnostic.

### C. One mistake, one diagnostic. A node that has an error is not checked again by later checks;
A node here is the smallest item the error is reported on: a name, a field, an attribute use, a declaration.

## Decision
Option C, a generalisation of B. The specification states once that a node with an error is not checked again by later checks, warnings included; the node is the smallest item the error is reported on. W0001 on a name with E0104 follows from it, with no special rule. E0104 is reported by the parser (phase 3).

## Consequences

A: change both E0104 cases (add `//~ W0001`), no spec change.
B: §3.2 or §12 gets one sentence naming the scope of the suppression; RFC §5.12 lists the poisoned name;
the check phase skips naming conventions for poisoned names. The cases stay as they are.

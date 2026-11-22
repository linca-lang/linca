# ADR-0001. Base of a `type` declaration in the grammar

- **Proposed:** 2026-11-22, by TeseySTD
- **Affects:** language
- **Spec:** Specification §4 (`newtype`), §5.5, §12 (E0103, E0221)
- **Cases:** tests/data/lang/diagnostics/E0221_type_base_not_scalar
- **Status:** accepted

## Context

§4 defines `newtype = "type" ident "=" type_name`, with the comment "base MUST be a built-in scalar".
§5.5 says `type Tags = List<String>` is E0221. With `type_name` in the grammar, `List<String>` does not
parse: the parser reports E0103 (phase 3) and the checker never sees the declaration. The two sections
cannot both hold. The case E0221_type_base_not_scalar uses `type RoleCode = Role`, which parses under both
options, so it does not decide the question. Affected: users writing `type` over a collection (which
code they see), and the parser/checker split.

## Options

### A. Keep `type_name` (E0103 for `List<...>`, `Map<...>`, `?`)
The grammar stays. A collection or nullable base is a syntax error. Consequences:
+ Grammar is exact; no change to §4. / + Parser is unchanged.
− §5.5 must be rewritten: E0221 covers only a named non-scalar (record, enum, another `type`).
− The message for `type Tags = List<String>` is "unexpected token", which does not explain the rule.

### B. Widen to `type_ref` (E0221 for everything that is not a scalar)
`newtype = "type" ident "=" type_ref`. The checker rejects collections, `T?` and non-scalar names with E0221.
Consequences:
+ §5.5 holds as written; one code and one message for the whole rule.
+ Better diagnostic: the checker can say "a domain type must be based on a built-in scalar".
− Grammar admits programs that are never valid; `type_ref` parsing is reachable from a `type` base.
− Milestone 1 does not report this error; it appears only in milestone 2 (phase 7). A new case with
  `List<String>` is needed to pin it.

## Decision
Option B. The parser accepts any type_ref as the base; the checker reports E0221 for everything that is not a built-in scalar, so the user sees the rule instead of a syntax error.

## Consequences

A: §5.5 changes; §4 unchanged; add a case `E0103` for `type Tags = List<String>`.
B: §4 changes (`newtype` base); add a case `E0221` for a collection base and one for `String?`;
the parser must not reject `?` itself. Either way the IR is unchanged.

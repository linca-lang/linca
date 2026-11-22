# ADR-0002. Parameter names in `attribute_decl`

- **Proposed:** 2026-11-22, by claude-sonnet-5-5
- **Affects:** language
- **Spec:** Specification §3.3, §4 (`attribute_decl`, `param`, `arg`), §7.4
- **Cases:** none yet (a case would go under tests/data/lang/valid/07_attributes)
- **Status:** accepted

## Context

§3.3 lets a keyword be a field name, a parameter name (of an operation) and an attribute **argument** name
without escaping: `@x(on: true)`. The grammar of `attribute_decl` uses `param = ident ":" type_ref ...`, so a
parameter of a user attribute cannot be called `on` or `type`. A call `@range(type: Int)` is valid by `arg`,
but the declaration of that parameter is a syntax error (E0103). The same names are allowed at the call
site and refused at the declaration site, so a plugin or user cannot declare the parameter it is
invited to pass. Affected: authors of user and plugin attributes.

## Options

### A. `ident` (keywords not allowed)
`param = ident ":" type_ref [ "=" value ]`. Consequences:

Positive:
- Grammar and §4 stay as they are; no ambiguity with `on` after the closing parenthesis.

Negative:
- Inconsistent with §3.3: `@a(on: true)` is syntactically allowed but cannot be declared.
- §3.3 should say that attribute argument names are limited to non-keywords in practice, or the
  call-site rule becomes dead for keywords.

### B. `field_name` (keywords allowed, except `true`, `false`, `null`)
`param = field_name ":" type_ref [ "=" value ]`. Consequences:

Positive:
- Matches `arg` and `op_param`; the rule "a keyword is a name before `:`" is uniform.
- Plugins can use natural names (`type`, `on`, `as`).

Negative:

- `attribute_decl` is `... "(" param ... ")" "on" target_list`: the parser must tell a parameter `on:` from
  the `on` after `)`. It can, because `on` after `)` is not followed by `:` and parameters are inside the
  parentheses, but this has to be checked in the parser.
- IR: the declared parameter name may be a keyword; generators must handle it (already true for fields).

## Decision
Option B. Lesser changes and much more profit for developers.

## Consequences

A: §3.3 gets one sentence on attribute parameters; §4 unchanged.
B: §4 changes `param`; §3.3 lists attribute parameters among the positions; add a valid case that declares
and uses a parameter named `type` and one named `on`. Plugin manifests are unaffected.

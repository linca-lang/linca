---
name: linca-spec
description: Map of the Linca specifications. It shows which spec section defines a rule or diagnostic code, which compiler phase and module own it, and where an IR key is defined. Use it before reading spec documents, so you open only the section you need.
user-invocable: false
---

Do not read whole documents. Find the section here, then open it with `grep -n "^### 9.2" docs/Specification.md`
and read that section only.

## Diagnostic codes → spec section → phase → module

| Codes | Topic | Specification | Phase (RFC §5) | Module |
|---|---|---|---|---|
| E0101–E0102 | lexical | §3 | 2 Lex | `lexer` |
| E0103–E0106 | syntax | §3.3, §4 | 3 Parse | `parser` |
| E0701–E0705, E0201, E0401, W0401, E0801–E0802 | names, imports | §10, §5, §7.1, §17.2–17.3 | 4 Resolve | `resolve` |
| E0410, E0404, E0402, E0601–E0603, E0304, E0303, E0810–E0811 | sets, mixins, method attributes | §7.4, §9.1, §6.2, §17.5 | 5 Expand | `expand` |
| E0610–E0613 | views, sensitive fields | §9.2 | 6 Views | `views` |
| E02xx (except E0201), E03xx (except E0303–E0304), E0403, E0406–E0409, W0405, E05xx, W0001 | types, declarations, attribute args, relations, naming | §5, §6, §7.2–7.7, §8, §3.2 | 7 Check | `check` |
| E0803–E0808, E0812–E0815 | services, HTTP binding | §17 | 8 Services | `services` |
| E0420–E0422 | capabilities | §16, §7.3, §7.6 | 9 Caps | `caps` |

The authoritative per-phase list is in Compiler RFC §5.2–§5.9. If it and this table disagree, the RFC wins.
Report the mismatch.

## Specification sections

§3 lexical (3.1 comments, 3.2 identifiers and naming, 3.3 contextual keywords, 3.4 literals) · §4 grammar (EBNF) ·
§5 types (5.1 scalars, 5.2 collections, 5.3 nullability vs presence, 5.4 nominal typing, 5.5 domain types,
5.6 where types may be used) · §6 declarations · §7 attributes (7.1 rules, 7.2 core table, 7.3 storage,
7.4 user attributes and sets, 7.5 plugin attributes, 7.6 generators, 7.7 argument binding and resolution) ·
§8 relations · §9 composition (9.1 mixins, 9.2 views) · §10 modules and names · §11 check order ·
§12 code table · §13 full example · §14 out of scope · §15 decisions and open questions · §16 capability-gated
constructs · §17 services (17.5 HTTP binding, 17.6 derived binding, 17.7 readonly, 17.8 sensitive data).

## IR Specification sections

§2 conventions (numbers as strings, ids, null vs []) · §3 document · §4 building blocks (4.1 Name and the
word-splitting rule, 4.2 Span, 4.3 TypeRef, 4.4 Value, 4.5 Constraints, 4.6 Origin, 4.7 Attribute) · §5 Field ·
§6 declarations (6.1 model … 6.9 service, operation, param, http binding) · §7 relations · §8 what is not in the IR ·
§9 canonical form · §10 versioning. The JSON Schema `schema/linca-ir.schema.json` is normative and wins over the text.

## Compiler RFC sections

§3 decisions table · §4 architecture and the two representations · §5 phases with their codes · §5.12 error
recovery and poisoning · §6 diagnostics structure and JSON output · §7 `linca.toml` · §8 CLI and exit codes ·
§9 generated files · §10 crates and modules · §11 tests · §13 milestones.

## Test cases

`tests/data/lang/README.md` defines how a case is compiled, the `//~` annotation rules and the chosen primary
lines where the spec is silent. The tables there are binding for the runner.

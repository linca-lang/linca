# Linca language test cases

Stage 1 of the language test cases for **Linca Core Language Specification 0.1** (draft of 2026-11-16).
It is written before the compiler exists, so it is a proposal for what the compiler should
accept and reject. When the compiler and a case disagree, one of three things is wrong:
the compiler, the case, or the specification. Decide which, and fix that one.

## Layout

```
lang/
  diagnostics/<CODE>_<name>/   one case per directory
      bad.linca  | bad/          must produce the annotated diagnostic
      ok.linca   | ok/           the passing twin: the smallest change that makes it valid
  valid/<NN_section>/          files and directories that must compile without diagnostics
  projects/<name>/             larger multi-module schemas, no diagnostics
  pending/                     cases that need plugin configuration; not runnable yet
  cases.json                   machine-readable index of every case
```

## How a case is compiled

- A single file (`bad.linca`, `ok.linca`, `valid/**/x.linca`) is compiled **alone**, as a schema
  with one module named after the file. Other files in the same directory are not part of it.
- A directory (`bad/`, `ok/`, `valid/10_modules/<name>/`, `projects/<name>/`) is a **schema
  root**. Module paths are relative to it: `geo/places.linca` is the module `geo::places`.
- No plugins are configured. Capability checks (step 8 of Section 11) therefore never fail,
  and every plugin attribute is W0401.

## Expectations

An expected diagnostic is a comment at the end of the line of its primary span:

```
x: Int,    //~ E0106
```

Annotations are found by scanning raw lines for `//~ <CODE>`, also inside block comments
(see E0102). A case passes when:

1. every annotated line has a diagnostic with that code whose primary span starts on it, and
2. the set of codes reported for the case equals the set of codes annotated.

A compiler MAY report the same code on further lines (E0502 on both ambiguous relations,
E0601 on both mixins of a cycle). Files without annotations must produce no diagnostics at
all, warnings included, so every valid file follows the naming conventions of Section 3.2.

Messages, secondary spans and fix suggestions are not asserted in stage 1.

## Where the specification does not fix the line

The specification names the code for each rule but rarely the primary span. Where it is
silent, this set of test cases picks a line; these picks are proposals to confirm:

| Code | Line chosen |
|---|---|
| E0220 | the foreign-key field, as in the comment of Section 13 |
| E0301 | the `model` line |
| E0302 | the second `@primary` field |
| E0321, E0322, E0323 | the `union` line |
| E0409 | the second `@rule` |
| E0410, E0601, E0612 | the first declaration of the cycle |
| E0501 | the first relation field of the pair |
| E0502, E0504, E0506, E0507 | the owning relation field |
| E0503 | the relation field |
| E0505 | the back-relation field |
| E0602 | the declaration with the `with` clause |
| E0702, E0704, E0705 | the `import` line |

## Services: where `//~` goes (temporary)

The specification does not define primary spans for the service codes yet. Until it does,
the cases put the annotation as follows. These picks are a temporary convention, to be
replaced when Section 12 or 17 fixes the spans.

| Code | Line chosen |
|---|---|
| E0104 on an operation | the operation line |
| E0403 on `@status` | the `@status` line |
| E0406 on a parameter default | the parameter line |
| E0801 | the name line of the second operation |
| E0802 | the line of the second parameter with the name |
| E0803 | the line of the field, parameter, result type or view |
| E0804 | the operation line, where `throws` is |
| E0805 | the operation line, where the second item of `throws` is |
| E0806 | the parameter line; the operation line for a result |
| E0807 | the operation line for a result; the field line for an error |
| E0808 | the field line |
| E0810 | the line of the method attribute that holds the path |
| E0811 | the line of the second method attribute |
| E0812 | the name line of the operation that lacks a parameter of the `@route` path; the line of the method attribute for a placeholder of its own path |
| E0813 | the parameter line |
| E0814 | the name line of the second operation |
| E0815 | the `@readonly` line |

## Stage 2

Once the IR has a draft, add `<name>.ir.json` next to every passing file: generate it with
the compiler, review it by hand, commit it as the expected output. Comments in `valid/`
files such as "field order: ..." say what the IR is expected to show.

## Header of every file

```
// Linca test case | spec 0.1 | section 9.2
// <what the case shows>
```

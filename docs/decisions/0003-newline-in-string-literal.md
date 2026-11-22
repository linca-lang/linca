# ADR-0003. A raw newline inside a string literal

- **Proposed:** 2026-11-22, by TeseySTD
- **Affects:** language
- **Spec:** Specification §3.4, §12 (E0102), Compiler RFC §5.2
- **Cases:** tests/data/lang/diagnostics/E0102_unterminated_comment (comment only; no case for strings)
- **Status:** accepted

## Context

§3.4 defines `string = '"' { char | escape } '"'` and does not say whether `char` may be a line feed.
§12 describes E0102 as "unterminated string or comment". A string with no closing quote runs to the end of
the file if newlines are allowed, and every following declaration is swallowed into it. Affected: users
(error quality after a missing quote, multi-line `@doc`-style text), and the lexer's recovery. The
`//~` annotation of a case is the line of the primary span, so the choice also fixes where tests put it.

## Options

### A. Allowed
A string may span lines; E0102 is reported only at end of file, with the span starting at the opening quote.
Consequences:

Positive:
- Long text (descriptions, SQL snippets in attributes) needs no `\n` escapes.

Negative:
- One forgotten quote turns the rest of the file into a string; the only diagnostic is E0102 at the
  opening quote, and all later errors vanish. Fixing it takes the user to a line far from the real mistake.
- Line numbers in later spans depend on string contents; harder for `linca fmt`.

### B. Not allowed (E0102 at the end of the line)
A raw LF or CR LF ends the string with E0102 whose primary span is the string starting at the opening quote
(reported on that line), and lexing continues on the next line. Consequences:

Positive:
- A missing quote damages one line; the rest of the file is still checked.
- Spans of strings are always single-line.

Negative:
- Multi-line text requires `\n` escapes or concatenation, which does not exist in the language.
- §3.4 must state the rule; E0102 is then "unterminated string" in the line sense and "unterminated comment"
  in the block sense.

## Decision
Option B. A raw line break ends the string with E0102 at the opening quote, and lexing resumes on the next line, so one missing quote damages one line. Multi-line text can be added later as a separate literal form without breaking existing files.

## Consequences

A: §3.4 says "a string may contain line breaks"; E0102 for strings is end-of-file only; add one case.
B: §3.4 gets "A string MUST NOT contain a raw line break (E0102)"; add a case under diagnostics/ for an
open string at the end of a line, with `//~` on that line. The lexer needs a recovery rule: resume after the line break.

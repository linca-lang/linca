# Decisions

The spec says *what* the language is; a decision record says *why*. When a spec rule follows from a
decision, the spec section names it as `ADR-NNNN`.

## Files

- `NNNN-kebab-title.draft.md`: a proposal, not reviewed yet. Agents may write only these.
- `NNNN-kebab-title.md`: a reviewed record with a `Status` line: `accepted`, `rejected` or
  `superseded by ADR-MMMM`. Only a human creates, renames or edits these.

Numbers are given in order when a draft is created, and are never changed or reused. Refer to a record as
`ADR-NNNN`, not by file path, because the file name changes on review.

## Reviewing a draft

1. Read the draft and its options.
2. Rename it: `git mv docs/decisions/0007-x.draft.md docs/decisions/0007-x.md`.
3. Add `- **Status:** accepted` (or `rejected`), and write the "Decision" section.
4. If accepted: change the spec and the cases in a spec session (`/spec-change ADR-0007`).
5. Commit.

A rejected record stays in the directory, so the same option is not proposed again.

The template is in `.claude/skills/adr/template.md`.
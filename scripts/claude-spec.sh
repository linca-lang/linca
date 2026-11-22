#!/bin/sh
# Start Claude Code in spec mode (spec, ADRs, test cases; compiler code locked).
# Usage, from the repository root:  scripts/claude-spec.sh [claude arguments]
LINCA_MODE=spec exec claude "$@"

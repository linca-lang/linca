#!/bin/sh
# Entry point for every Linca hook. Claude Code runs hook commands with sh on Linux and macOS
# and with Git Bash on Windows, so this file is plain POSIX sh.
# It runs linca_guard.py with the first working Python 3.9+:
#   python3 (Linux, macOS), py (Windows launcher), python (Windows).
# The probe skips the Microsoft Store "python" stubs, which exist but do not run.
dir=${0%/*}  # directory of this script, without calling dirname
for py in python3 py python; do
  if command -v "$py" >/dev/null 2>&1 &&
     "$py" -c "import sys; sys.exit(0 if sys.version_info >= (3, 9) else 1)" >/dev/null 2>&1 </dev/null; then
    exec "$py" "$dir/linca_guard.py" "$@"
  fi
done
echo "Linca hooks: no Python 3.9+ found (tried python3, py, python). Every tool call is blocked until one is installed." >&2
exit 2

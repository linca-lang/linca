#!/usr/bin/env python3
"""Guard for Claude Code sessions in the Linca repository. Runs on Linux, macOS and Windows.

Started through guard.sh, which picks the Python interpreter. Five hook events (see .claude/settings.json):

  session-start  print the mode into the session; snapshot the locked files
  session-end    delete the session's snapshot
  pre-tool       block edits to files the current mode locks (exit 2)
  post-edit      rustfmt an edited .rs file
  stop           refuse to finish while a locked file differs from the snapshot

Modes, chosen when Claude Code is started (scripts/claude-spec.sh or .ps1 start a spec session):

  impl (default)    write compiler code; spec, IR, tests, harness are locked
  LINCA_MODE=spec   write spec, test cases; compiler code is locked
  both modes        ADR drafts (docs/decisions/NNNN-title.draft.md) are open; reviewed ADRs are locked

The point is the rule "whoever writes a test does not write the code that passes it".
"""
import hashlib
import json
import os
import posixpath
import re
import subprocess
import sys
from pathlib import Path

WINDOWS = os.name == "nt"


def native_path(p):
    """Git Bash may hand over /c/Users/... paths; Windows Python needs C:/Users/..."""
    p = str(p)
    if WINDOWS:
        m = re.match(r"^/([A-Za-z])(/.*)?$", p)
        if m:
            p = f"{m.group(1)}:{m.group(2) or '/'}"
    return Path(p)


ROOT = native_path(os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd()).resolve()
MODE = os.environ.get("LINCA_MODE", "impl")

# Prefixes relative to the repository root, with forward slashes.
LOCKED = {
    "impl": [
        "docs/", "schema/", "tests/data/lang/", "examples/",
        "AGENTS.md", "CLAUDE.md", ".gitattributes",
    ],
    "spec": ["crates/", "Cargo.toml", "Cargo.lock"],
}
HARNESS = [".claude/", "AI-POLICY.md", "docs/ai-workflow.md"] # edited by hand only
# Final decision records belong to the human. An agent writes only drafts, NNNN-kebab-title.draft.md;
# the human reviews a draft by renaming it to NNNN-kebab-title.md and setting its status.
DECISIONS = "docs/decisions/"
DRAFT_ADR = r"docs/decisions/\d{4}-[a-z0-9]+(?:-[a-z0-9]+)*\.draft\.md"
ALWAYS_LOCKED = HARNESS + [DECISIONS]

# Shell commands that write. Bash and Git Bash first, then PowerShell cmdlets and aliases.
WRITE_VERBS = (
    r"\b(?:mv|cp|rm|touch|truncate|ln|install|rsync|tee|chmod|del|erase|move|copy|ren"
    r"|set-content|add-content|out-file|remove-item|move-item|copy-item|new-item|rename-item"
    r"|clear-content|sc|ac|ni|ri|mi|cpi)\b"
)
INPLACE = r"\b(?:sed|perl)\b[^|;&]*\s-i"
GIT_TREE = r"\bgit\s+(?:checkout|restore|rm|mv|apply)\b"
# Git commands that throw away working-tree changes everywhere, locked files included, without naming a path.
GIT_DISCARD = r"\bgit\s+(?:stash(?!\s+(?:list|show)\b)|clean\b|reset\s+[^|;&]*--hard)"
DOTNET_WRITE = r"\[(?:system\.)?io\.file\]::(?:write|append|delete|move|copy|create)"

BASELINE_DIR = ROOT / ".claude" / ".baseline"


def fail(msg):
    print(msg, file=sys.stderr)
    sys.exit(2)


def norm(s):
    """Compare paths with forward slashes, and case-insensitively on Windows."""
    s = re.sub(r"[\\/]+", "/", s)  # tests\data, tests\\data and tests//data all mean tests/data
    return s.lower() if WINDOWS else s


def locked_prefixes():
    if MODE not in LOCKED:
        fail(f"LINCA_MODE={MODE!r} is unknown; use impl or spec")
    return LOCKED[MODE] + ALWAYS_LOCKED


def is_locked(rel):
    r = norm(rel)
    if re.fullmatch(DRAFT_ADR, r):  # an ADR draft is open in every mode
        return False
    return any(r == norm(p) or r.startswith(norm(p)) for p in locked_prefixes())


def rel_path(path):
    p = native_path(path)
    if not p.is_absolute():
        p = Path(os.getcwd()) / p
    try:
        return p.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return None  # outside the repository


def explain(what):
    if norm(what).startswith(norm(DECISIONS)):
        return (f"Blocked: {what} is a reviewed decision record, or not a valid draft name. Agents write only "
                "drafts named docs/decisions/NNNN-kebab-title.draft.md (adr skill). Renaming a draft or editing "
                "a reviewed record is the human's decision.")
    if any(norm(p) in norm(what) for p in HARNESS):
        return f"Blocked: {what} is part of the harness and is edited by hand only, in every mode."
    if MODE == "impl":
        return (f"Blocked in impl mode: {what} is locked. Do not change the spec, the IR schema, the test "
                "cases or the harness to make code pass. If you think a case or the spec is wrong, write an "
                "ADR draft with the adr skill (docs/decisions/NNNN-title.draft.md) and stop.")
    return f"Blocked in spec mode: {what} is locked. Spec sessions do not write compiler code."


# --- pre-tool ---------------------------------------------------------------

# A path-like word in a shell command: no spaces, quotes or shell operators.
TOKEN = re.compile(r"[^\s'\"`;|&<>()=,]+")
TOKEN_END = r"(?=$|[\s'\"`;|&<>(),])"
TOKEN_START = r"(?:^|(?<=[\s'\"`;|&<>(=,]))"
# Words that name the whole tree: rm -rf ., git restore ., rm -rf *, git checkout -- :/
WHOLE_TREE = [r"\.", r"\*+", r":/?"]


def root_spellings():
    """The repository root as it can appear in a command, already normalised."""
    r = norm(str(ROOT)).rstrip("/")
    spellings = {r}
    if WINDOWS and re.match(r"^[a-z]:/", r):
        spellings.add(f"/{r[0]}{r[2:]}")  # Git Bash: /c/Users/... for C:/Users/...
    return spellings


def norm_command(cmd):
    """Normalise every path-like word: separators, case on Windows, the repository root
    written out in full, ./ and .. segments, trailing slashes. Then "rm -rf C:\\repo\\tests\\data\\lang\\",
    "rm -rf ./crates/../tests/data/lang" and "rm -rf tests/data/lang" all read "rm -rf tests/data/lang"."""
    roots = root_spellings()

    def fix(m):
        t = m.group(0)
        for r in roots:
            if t == r:
                t = "."
            elif t.startswith(r + "/"):
                t = t[len(r) + 1:]
        if "/" in t and not t.startswith(":"):
            t = posixpath.normpath(t)
        return t

    return TOKEN.sub(fix, norm(cmd))


def shell_targets():
    """Regex alternatives for words that reach a locked file: the locked path itself or anything in it,
    a parent directory of it named as a whole (rm -rf tests, rm -rf tests/*), or the whole tree."""
    alts = list(WHOLE_TREE)
    for prefix in locked_prefixes():
        p = norm(prefix).rstrip("/")
        alts.append(rf"{re.escape(p)}(?:/[^\s'\"`;|&<>(),]*)?")
        parts = p.split("/")
        for i in range(1, len(parts)):
            alts.append(rf"{re.escape('/'.join(parts[:i]))}(?:/\*+)?")
    return rf"{TOKEN_START}['\"]?(?:{'|'.join(alts)}){TOKEN_END}"


def check_shell(cmd):
    if MODE == "impl" and "LINCA_BLESS" in cmd.upper():
        fail("Blocked: blessing expected output (LINCA_BLESS) is for the human, after review.")
    s = norm_command(cmd)
    s = re.sub(rf"{TOKEN_START}{DRAFT_ADR}{TOKEN_END}", "OPEN", s)  # ADR drafts are fine
    if re.search(GIT_DISCARD, s):
        fail("Blocked: git stash, git clean and git reset --hard discard changes in the whole tree, "
             "including the human's uncommitted changes to locked files. Undo your own edits file by file.")
    target = shell_targets()
    flags = re.IGNORECASE
    if (
        re.search(rf">>?\s*{target}", s)
        or re.search(rf"{WRITE_VERBS}[^|;&]*{target}", s, flags)
        or re.search(rf"{INPLACE}[^|;&]*{target}", s)
        or re.search(rf"{GIT_TREE}[^|;&]*{target}", s)
        or re.search(rf"{DOTNET_WRITE}[^|;&]*{target}", s, flags)
    ):
        fail(explain("a shell command that writes to a locked path (or to a directory that contains one)"))


def pre_tool(data):
    tool = data.get("tool_name", "")
    inp = data.get("tool_input", {}) or {}
    if tool in ("Bash", "PowerShell"):
        check_shell(inp.get("command", ""))
        return
    path = inp.get("file_path") or inp.get("notebook_path")
    if not path:
        return
    rel = rel_path(path)
    if rel and is_locked(rel):
        fail(explain(rel))


# --- snapshot of locked files -----------------------------------------------
#
# The snapshot does not hash every locked file. Git already knows what differs from HEAD, so a
# snapshot holds only HEAD and the hashes of the locked files that were already dirty at session
# start (usually none). Size and time do not grow with the number of test cases.

BASELINE_MAX_AGE = 7 * 24 * 3600  # remove snapshots left by sessions that never sent SessionEnd


def git(*args):
    try:
        r = subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True)
    except FileNotFoundError:
        return None
    return r.stdout.decode("utf-8", "replace") if r.returncode == 0 else None


def pathspecs():
    specs = [p.rstrip("/") for p in locked_prefixes()]
    return ["--", *specs]


def file_hash(rel):
    f = ROOT / rel
    return hashlib.sha256(f.read_bytes()).hexdigest() if f.is_file() else "missing"


def dirty_locked():
    """Locked files that differ from HEAD: modified, added, deleted or untracked."""
    out = git("status", "--porcelain=v1", "-z", "--untracked-files=all", "--no-renames", *pathspecs())
    if out is None:
        return None
    files = set()
    for entry in out.split("\0"):
        if len(entry) > 3:
            rel = entry[3:]
            if is_locked(rel):
                files.add(rel)
    return files


def snapshot():
    dirty = dirty_locked()
    if dirty is None:
        return None
    head = (git("rev-parse", "--verify", "-q", "HEAD") or "").strip() or None
    return {"head": head, "dirty": {f: file_hash(f) for f in sorted(dirty)}}


def changed_since(snap):
    """Locked files whose content now differs from the snapshot."""
    dirty_now = dirty_locked()
    if dirty_now is None:
        return []
    before = snap["dirty"]
    changed = {f for f in dirty_now if f not in before}            # clean at start, dirty now
    changed |= {f for f in before if file_hash(f) != before[f]}     # dirty at start, changed again
    head = (git("rev-parse", "--verify", "-q", "HEAD") or "").strip() or None
    if snap["head"] and head and head != snap["head"]:
        # Committed during the session: the tree can be clean and still differ from the start.
        out = git("diff", "--name-only", "-z", snap["head"], head, *pathspecs()) or ""
        changed |= {f for f in out.split("\0") if f and is_locked(f) and f not in before}
    return sorted(changed)


def baseline_path(data):
    sid = re.sub(r"[^A-Za-z0-9_-]", "", data.get("session_id", "") or "") or "default"
    return BASELINE_DIR / f"{sid}-{MODE}.json"


def prune_baselines():
    import time
    for f in BASELINE_DIR.glob("*.json"):
        try:
            if time.time() - f.stat().st_mtime > BASELINE_MAX_AGE:
                f.unlink()
        except OSError:
            pass


def session_start(data):
    BASELINE_DIR.mkdir(parents=True, exist_ok=True)
    prune_baselines()
    snap = snapshot()
    warn = ""
    if snap is None:
        warn = " WARNING: git is not available, so the end-of-turn check of locked files is off."
    else:
        baseline_path(data).write_text(json.dumps(snap), encoding="utf-8")
    locked = ", ".join(LOCKED[MODE])
    if MODE == "impl":
        hint = ("Write compiler code; never change spec, schema, cases or harness. "
                "Use /implement-cases for a task; use the adr skill when a case or the spec looks wrong.")
    else:
        hint = "Write spec, ADRs and test cases; do not write compiler code. Use /spec-change."
    print(f"Linca session mode: {MODE}. Locked in this mode: {locked}. {hint}{warn}")


def session_end(data):
    try:
        baseline_path(data).unlink()
    except OSError:
        pass


def stop(data):
    path = baseline_path(data)
    if not path.exists():
        return
    snap = json.loads(path.read_text(encoding="utf-8"))
    if "dirty" not in snap:  # snapshot from the old format: start over
        snap = snapshot()
        if snap:
            path.write_text(json.dumps(snap), encoding="utf-8")
        return
    changed = changed_since(snap)
    if not changed:
        return
    listing = ", ".join(changed[:20]) + (" …" if len(changed) > 20 else "")
    if data.get("stop_hook_active"):
        # Already asked once; do not loop. Tell the human, then accept the current state,
        # so the same change is not reported again on every later turn.
        print(json.dumps({"systemMessage": f"Locked files changed in {MODE} mode: {listing}"}))
        new = snapshot()
        if new:
            path.write_text(json.dumps(new), encoding="utf-8")
        return
    fail(f"Locked files changed during this {MODE} session: {listing}. "
         "If you changed them, restore them to their state at session start (git diff shows what changed), "
         "or explain in your report why you could not. "
         "If someone else changed them, do not touch anything; just list them in your report.")


# --- post-edit ----------------------------------------------------------------

def post_edit(data):
    path = (data.get("tool_input", {}) or {}).get("file_path")
    rel = rel_path(path) if path else None
    if not rel:
        return
    if rel.endswith(".rs"):
        try:
            subprocess.run(["rustfmt", "--edition", "2021", str(ROOT / rel)],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        except FileNotFoundError:
            pass


def main():
    event = sys.argv[1] if len(sys.argv) > 1 else ""
    try:
        data = json.loads(sys.stdin.buffer.read().decode("utf-8") or "{}")
    except ValueError:
        data = {}
    handlers = {"session-start": session_start, "session-end": session_end, "pre-tool": pre_tool,
                "post-edit": post_edit, "stop": stop}
    handlers.get(event, lambda _: None)(data)


if __name__ == "__main__":
    main()
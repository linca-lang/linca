# Start Claude Code in spec mode (spec, test cases, ADR drafts; compiler code locked).
# Usage, from the repository root:  nu scripts/claude-spec.nu [claude arguments]
#   e.g.  nu scripts/claude-spec.nu --resume
# LINCA_MODE is set for this one run only, so a later plain `claude` is impl mode again.
# --wrapped passes flags such as --resume on to claude instead of parsing them here.
def --wrapped main [...args] {
    with-env { LINCA_MODE: "spec" } {
        ^claude ...$args
    }
}
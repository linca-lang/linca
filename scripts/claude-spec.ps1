# Start Claude Code in spec mode (spec, ADRs, test cases; compiler code locked).
# Usage, from the repository root:  .\scripts\claude-spec.ps1 [claude arguments]
# LINCA_MODE is set for this one run only, so a later plain `claude` is impl mode again.
$previous = $env:LINCA_MODE
$env:LINCA_MODE = "spec"
try {
    claude @args
}
finally {
    if ($null -eq $previous) { Remove-Item Env:LINCA_MODE -ErrorAction SilentlyContinue }
    else { $env:LINCA_MODE = $previous }
}

$ErrorActionPreference = 'Stop'

$leanTargetPath = Join-Path (Split-Path -Parent $PSScriptRoot) 'target'
if (-not (Test-Path -LiteralPath $leanTargetPath)) {
    Write-Output 'target does not exist'
    return
}

$leanTargetBytes = (Get-ChildItem -LiteralPath $leanTargetPath -Recurse -File |
    Measure-Object -Property Length -Sum).Sum
Write-Output ('target: {0:N1} MiB' -f ($leanTargetBytes / 1MB))

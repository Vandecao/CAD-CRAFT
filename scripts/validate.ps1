[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
Push-Location (Split-Path $PSScriptRoot -Parent)
try {
    $checks = @(
        @{ Tool = 'cargo'; Arguments = @('fmt', '--check') },
        @{ Tool = 'cargo'; Arguments = @('check', '--workspace', '--all-targets', '--locked') },
        @{ Tool = 'cargo'; Arguments = @('test', '--workspace', '--locked') },
        @{ Tool = 'cargo'; Arguments = @('test', '-p', 'cad-craft', '--examples', '--locked') },
        @{ Tool = 'cargo'; Arguments = @('clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings') },
        @{ Tool = 'pwsh'; Arguments = @('-NoProfile', '-File', 'scripts/tests/check-repository.Tests.ps1') },
        @{ Tool = 'git'; Arguments = @('diff', '--check') },
        @{ Tool = 'git'; Arguments = @('diff', '--cached', '--check') }
    )
    foreach ($check in $checks) {
        Write-Host ("{0} {1}" -f $check.Tool, ($check.Arguments -join ' '))
        & $check.Tool @($check.Arguments)
        if ($LASTEXITCODE -ne 0) {
            throw "Validação falhou: $($check.Tool) (exit $LASTEXITCODE)."
        }
    }
} finally {
    Pop-Location
}

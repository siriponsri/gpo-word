param([switch]$Check)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location -LiteralPath $projectRoot
try {
    $buildRoot = if ($env:CARGO_TARGET_DIR) { [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR) } else { Join-Path $projectRoot 'target' }
    $candidates = @((Join-Path $projectRoot 'GPOWord.exe'),(Join-Path $buildRoot 'release\wordcraft.exe'))
    if (-not $Check) {
        foreach ($candidate in $candidates) {
            if (Test-Path -LiteralPath $candidate) {
                Start-Process -FilePath $candidate -WorkingDirectory $projectRoot
                return
            }
        }
    }
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw 'This operation requires Rust 1.95+ and a Windows C/C++ toolchain. To use a built portable package, open its GPOWord.exe directly.'
    }
    if ($Check) {
        cargo test -p wordcraft-engine --test gpo
        if ($LASTEXITCODE -ne 0) { throw 'GPO integration tests failed' }
        cargo xtask ci
        if ($LASTEXITCODE -ne 0) { throw 'Required repository checks failed' }
    } else {
        cargo run --release -p wordcraft
        if ($LASTEXITCODE -ne 0) { throw 'Build or launch failed' }
    }
} finally { Pop-Location }

param([string]$Templates,[string]$OutputDirectory,[string]$BinaryDirectory,[switch]$IncludeSource)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
if (-not $Templates) { $Templates = Join-Path $projectRoot 'docsStorage' }
if (-not $OutputDirectory) { $OutputDirectory = Join-Path $projectRoot 'dist\GPOWord-portable' }
if (-not $BinaryDirectory) {
    $buildRoot = if ($env:CARGO_TARGET_DIR) { [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR) } else { Join-Path $projectRoot 'target' }
    $BinaryDirectory = Join-Path $buildRoot 'release'
}
$resolvedProject = [IO.Path]::GetFullPath($projectRoot)
$resolvedOutput = [IO.Path]::GetFullPath($OutputDirectory)
$resolvedTemplates = [IO.Path]::GetFullPath($Templates)
$separator = [IO.Path]::DirectorySeparatorChar
if ($resolvedOutput.StartsWith($resolvedTemplates + $separator,[StringComparison]::OrdinalIgnoreCase) -or $resolvedOutput -eq $resolvedTemplates) {
    throw 'Output must not be inside the template folder'
}
if ($resolvedOutput.StartsWith($resolvedProject + $separator,[StringComparison]::OrdinalIgnoreCase)) {
    $relativeOutput = $resolvedOutput.Substring($resolvedProject.Length + 1)
    if (($relativeOutput -split '[\\/]')[0] -ne 'dist') { throw 'An output inside the source project must be under dist to prevent recursive copying' }
}
$binaryPath = Join-Path $BinaryDirectory 'wordcraft.exe'
if (-not (Test-Path -LiteralPath $binaryPath)) { throw 'Build the real Windows EXE first. A source package is not a runnable release.' }
if (-not (Test-Path -LiteralPath (Join-Path $Templates 'catalog.json'))) { throw 'Select a docsStorage template pack containing catalog.json' }
if ((Test-Path -LiteralPath $OutputDirectory) -or (Test-Path -LiteralPath "$OutputDirectory.zip")) { throw 'Choose a new output path; existing packages will not be overwritten' }
New-Item -ItemType Directory -Path $OutputDirectory | Out-Null
if ($IncludeSource) {
    $excludedTop = @('.git','docsStorage','private','data','documents','target','dist','upstreams','node_modules')
    foreach ($file in (Get-ChildItem -LiteralPath $projectRoot -Recurse -File -Force)) {
        $relative = $file.FullName.Substring($projectRoot.Length + 1)
        $top = ($relative -split '[\\/]')[0]
        if ($top -in $excludedTop -or $relative -match '^samples[\\/]private[\\/]') { continue }
        if ($file.Name -match '^\.env($|\.)' -or $file.Extension -in @('.exe','.dll','.dpapi','.pfx','.p12','.key','.tmp')) { continue }
        if ($relative -in @('MANIFEST.json','ARCHIVE-AUDIT.json','GPOWord.portable')) { continue }
        $destination = Join-Path $OutputDirectory $relative
        New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force | Out-Null
        Copy-Item -LiteralPath $file.FullName -Destination $destination
    }
} else {
    foreach ($name in @('LICENSE-MIT','LICENSE-APACHE','NOTICE','ATTRIBUTION.md','VALIDATION.md','THIRD-PARTY-NOTICES.txt')) {
        $path = Join-Path $projectRoot $name
        if (Test-Path -LiteralPath $path) { Copy-Item -LiteralPath $path -Destination $OutputDirectory }
    }
    foreach ($relative in @('gpo','docs\manual','assets\brand')) {
        $path = Join-Path $projectRoot $relative
        if (Test-Path -LiteralPath $path) {
            $destination = Join-Path $OutputDirectory $relative
            New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force | Out-Null
            Copy-Item -LiteralPath $path -Destination $destination -Recurse
        }
    }
    $readme = @'
# GPO Word portable preview

Extract the whole package and open GPOWord.exe. Rust and C++ build tools are not needed on the recipient PC.

Read the [Thai user guide](docs/manual/th/index.html) in a browser, or the [Markdown guide](docs/manual/th/README.md).

The private template pack is in docsStorage. Each user supplies their own API key in Settings. No previous user's key or data is included.

See VALIDATION.md for actual checks and limitations. This preview does not establish document approval or GMP compliance.
'@
    [IO.File]::WriteAllText((Join-Path $OutputDirectory 'README.md'),$readme,(New-Object Text.UTF8Encoding($false)))
}
Copy-Item -LiteralPath $binaryPath -Destination (Join-Path $OutputDirectory 'GPOWord.exe')
$cliPath = Join-Path $BinaryDirectory 'wordcraft-cli.exe'
if (Test-Path -LiteralPath $cliPath) { Copy-Item -LiteralPath $cliPath -Destination $OutputDirectory }
foreach ($dll in (Get-ChildItem -LiteralPath $BinaryDirectory -Filter '*.dll' -File)) {
    Copy-Item -LiteralPath $dll.FullName -Destination $OutputDirectory
}
Copy-Item -LiteralPath $Templates -Destination (Join-Path $OutputDirectory 'docsStorage') -Recurse
Set-Content -LiteralPath (Join-Path $OutputDirectory 'GPOWord.portable') -Value '1' -Encoding ascii
New-Item -ItemType Directory -Path (Join-Path $OutputDirectory 'data'),(Join-Path $OutputDirectory 'documents') | Out-Null
Compress-Archive -LiteralPath $OutputDirectory -DestinationPath "$OutputDirectory.zip"
Write-Output "Portable package: $OutputDirectory.zip. Existing user data and API keys were excluded."

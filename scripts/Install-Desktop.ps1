param([string]$Destination)
$ErrorActionPreference = 'Stop'
$sourceRoot = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
if (-not $Destination) {
    $Destination = Join-Path ([Environment]::GetFolderPath('Desktop')) 'myProject\gpo-word'
}
$targetRoot = [IO.Path]::GetFullPath($Destination)
if ($targetRoot -eq $sourceRoot) {
    Write-Output "Project is already at $targetRoot"
    return
}
if ($targetRoot.StartsWith($sourceRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Destination must not be inside the source folder'
}
if ([IO.Path]::GetPathRoot($targetRoot) -eq $targetRoot) { throw 'A drive root is not a project destination' }
if (Test-Path -LiteralPath (Join-Path $targetRoot '.git')) {
    throw 'Destination already has Git history. Review that working tree before copying a source package into it.'
}
$excludedTop = @('.git','target','dist','data','documents')
$sourceFiles = Get-ChildItem -LiteralPath $sourceRoot -Recurse -File -Force | Where-Object {
    $relative = $_.FullName.Substring($sourceRoot.Length + 1)
    $top = ($relative -split '[\\/]')[0]
    $top -notin $excludedTop
}
$conflicts = @()
foreach ($file in $sourceFiles) {
    $relative = $file.FullName.Substring($sourceRoot.Length + 1)
    $target = Join-Path $targetRoot $relative
    if ($relative -eq '.gitignore') { continue }
    if (Test-Path -LiteralPath $target) {
        if ((Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash -ne (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash) {
            $conflicts += $relative
        }
    }
}
if ($conflicts.Count -gt 0) {
    throw ("Existing files differ; no project files copied. Review these paths first: " + ($conflicts -join ', '))
}
New-Item -ItemType Directory -Path $targetRoot -Force | Out-Null
foreach ($file in $sourceFiles) {
    $relative = $file.FullName.Substring($sourceRoot.Length + 1)
    if ($relative -eq '.gitignore') { continue }
    $target = Join-Path $targetRoot $relative
    if (-not (Test-Path -LiteralPath $target)) {
        New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force | Out-Null
        Copy-Item -LiteralPath $file.FullName -Destination $target
    }
}
$ignoreLines = @()
$targetIgnore = Join-Path $targetRoot '.gitignore'
if (Test-Path -LiteralPath $targetIgnore) { $ignoreLines += Get-Content -LiteralPath $targetIgnore -Encoding UTF8 }
$ignoreLines += Get-Content -LiteralPath (Join-Path $sourceRoot '.gitignore') -Encoding UTF8
$ignoreLines += 'samples/private/'
$ignoreLines += '/GPOWord.exe'
$ignoreLines += '/wordcraft-cli.exe'
$ignoreLines += '/*.dll'
$ignoreLines += '/GPOWord.portable'
[IO.File]::WriteAllLines($targetIgnore, ($ignoreLines | Select-Object -Unique), (New-Object Text.UTF8Encoding($false)))
Write-Output "Project copied to $targetRoot"
Write-Output 'Existing samples and unrelated files were preserved. No Git push was performed.'

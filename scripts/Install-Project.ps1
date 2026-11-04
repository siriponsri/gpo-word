param([string]$Destination)
$ErrorActionPreference = 'Stop'
if (-not $Destination) { $Destination = Join-Path ([Environment]::GetFolderPath('Desktop')) 'myProject\gpoDocs' }
$sourceRoot = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$targetRoot = [IO.Path]::GetFullPath($Destination)
if ($sourceRoot -eq $targetRoot) { throw 'Source and destination must differ' }
if ($targetRoot.StartsWith($sourceRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Destination must not be inside the source project folder' }
if (Test-Path -LiteralPath (Join-Path $targetRoot '.git')) { throw 'Destination already contains a Git repository; review it before installing' }
New-Item -ItemType Directory -Path $targetRoot -Force | Out-Null
$excluded = @('docsStorage','private','target','upstreams')
$entries = Get-ChildItem -LiteralPath $sourceRoot -Force | Where-Object { $_.Name -notin $excluded }
foreach ($entry in $entries) {
    $target = Join-Path $targetRoot $entry.Name
    if (Test-Path -LiteralPath $target) { throw "Existing project path will not be overwritten: $target" }
}
foreach ($entry in $entries) { Copy-Item -LiteralPath $entry.FullName -Destination $targetRoot -Recurse -Force }
$sourceTemplates = Join-Path $sourceRoot 'docsStorage'
$targetTemplates = Join-Path $targetRoot 'docsStorage'
if ((Test-Path -LiteralPath $sourceTemplates) -and -not (Test-Path -LiteralPath $targetTemplates)) {
    Copy-Item -LiteralPath $sourceTemplates -Destination $targetTemplates -Recurse
}
$snapshotParent = Split-Path -Parent $sourceRoot
foreach ($upstreamName in @('wordcraft','khiankrub')) {
    $snapshot = Join-Path $snapshotParent "$upstreamName-upstream"
    $upstreamTarget = Join-Path $targetRoot "upstreams\$upstreamName"
    if ((Test-Path -LiteralPath $snapshot) -and -not (Test-Path -LiteralPath $upstreamTarget)) {
        New-Item -ItemType Directory -Path (Split-Path -Parent $upstreamTarget) -Force | Out-Null
        Copy-Item -LiteralPath $snapshot -Destination $upstreamTarget -Recurse -Force
    }
}
Write-Output "Project installed at $targetRoot. Existing docsStorage was preserved."

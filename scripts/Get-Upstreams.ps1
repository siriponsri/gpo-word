param([string]$Directory)
$ErrorActionPreference = 'Stop'
if (-not $Directory) { $Directory = Join-Path (Split-Path -Parent $PSScriptRoot) 'upstreams' }
New-Item -ItemType Directory -Path $Directory -Force | Out-Null
foreach ($item in @(@{Name='wordcraft';Url='https://github.com/storytold/wordcraft.git';Ref='fd7f6621a1419c7c99f2293c9e99b1b7c4b87282'},@{Name='khiankrub';Url='https://github.com/chanonss/khiankrub.git';Ref='5ae5731ff8e343f26ec35b485c16b8b99af63662'})) {
    $path = Join-Path $Directory $item.Name
    if (Test-Path -LiteralPath $path) { Write-Output "Keeping existing checkout: $path"; continue }
    git clone --no-checkout $item.Url $path
    if ($LASTEXITCODE -ne 0) { throw "Cannot clone $($item.Name)" }
    git -C $path checkout --detach $item.Ref
    if ($LASTEXITCODE -ne 0) { throw "Cannot check out pinned $($item.Name)" }
}

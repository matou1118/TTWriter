<#
.SYNOPSIS
  Cut a release: set the version, build, package it and write the update manifest.

.DESCRIPTION
  The version lives in one place - [workspace.package] version in the root Cargo.toml - because
  it is what the binary reports (About box, `wordcraft-cli --version`) and what the updater
  compares against the release tag. If the two drift, the updater either never offers an
  update or offers the wrong build, so this script writes the version there first, then runs
  the packaging (build, sign, pack, manifest) and prints the publishing steps.

  The MSI's ProductVersion stays the numeric major.minor.build, so a pre-release suffix
  (0.9.0-zh.3) does not reach the installer.

.EXAMPLE
  pwsh packaging/windows/release.ps1 -Version 0.9.0-zh.3
#>
param(
  [Parameter(Mandatory = $true)] [string] $Version,
  [ValidateSet('x64', 'x86', 'arm64')] [string] $Arch = 'x64',
  [switch] $SkipBuild
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

if ($Version -notmatch '^\d+\.\d+\.\d+(-[0-9A-Za-z.]+)?$') { throw "not a version: $Version" }
$Tag = "v$Version"

# ---- the version into Cargo.toml ------------------------------------------------------------------
$CargoToml = Join-Path $Root 'Cargo.toml'
$lines = [System.Collections.Generic.List[string]](Get-Content $CargoToml)
$inPkg = $false
$changed = $false
for ($i = 0; $i -lt $lines.Count; $i++) {
  if ($lines[$i] -match '^\s*\[') { $inPkg = ($lines[$i].Trim() -eq '[workspace.package]'); continue }
  if ($inPkg -and $lines[$i] -match '^\s*version\s*=\s*"([^"]+)"') {
    if ($Matches[1] -ne $Version) {
      $lines[$i] = $lines[$i] -replace '"[^"]+"', "`"$Version`""
      $changed = $true
    }
    break
  }
}
if (-not $changed -and $lines -notcontains "`$version = `"$Version`"") { throw 'no [workspace.package] version in Cargo.toml' }
[System.IO.File]::WriteAllLines($CargoToml, $lines, (New-Object System.Text.UTF8Encoding($false)))
Write-Output "==> Cargo.toml version: $Version"

# ---- build, sign, pack, manifest -----------------------------------------------------------------
$env:WORDCRAFT_VERSION = $Version
& (Join-Path $PSScriptRoot 'package.ps1') -Arch $Arch -SkipBuild:$SkipBuild
if ($LASTEXITCODE -ne 0) { throw "packaging failed with exit code $LASTEXITCODE" }

# ---- what to publish -----------------------------------------------------------------------------
$Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist\release' }
$manifest = Get-Content (Join-Path $Dist 'latest.json') -Raw | ConvertFrom-Json
$assets = $manifest.assets | Where-Object { $_.channel -eq 'cn' }

Write-Output ''
Write-Output "==> release $Tag"
Write-Output ''
Write-Output '  1. commit the version and the manifest'
Write-Output "       git add Cargo.toml site/latest.json"
Write-Output "       git commit -m `"release: $Version`""
Write-Output "       git tag $Tag"
Write-Output '       git push atomgit master; git push origin master; git push atomgit $Tag; git push origin $Tag'
Write-Output ''
Write-Output '  2. upload the assets to the release (both hosts)'
foreach ($a in $assets) {
  $file = Join-Path $Dist $a.name
  Write-Output "       gh release create $Tag `"$file`" --repo matou1118/ttwriter --title `"$Tag`" --notes `"$Version`""
  Write-Output "       # atomgit: POST /api/v5/repos/matou118/ttwriter/releases/$Tag/assets (needs your token)"
}
Write-Output ''
Write-Output '  3. the manifest the updater reads'
Write-Output "       site/latest.json  ->  https://matou1118.github.io/ttwriter/latest.json"
Write-Output ''
Write-Output '     sha256:'
foreach ($a in $assets) { Write-Output ("       {0,-46} {1}" -f $a.name, $a.sha256) }

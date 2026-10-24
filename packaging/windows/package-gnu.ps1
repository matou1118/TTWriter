<#
.SYNOPSIS
  Build and package WordCraft (TTWriter) for Windows with the GNU toolchain.

.DESCRIPTION
  This machine's MSVC linker is unavailable, so releases are built with the
  x86_64-pc-windows-gnu toolchain instead. This script is the GNU twin of
  package.ps1: it builds both binaries and produces the portable zip. The
  MSI installer needs WiX + MSVC tooling and stays out of scope here.

  Produces, in $env:DIST (default: dist\release-gnu):
    wordcraft-<version>-windows-x64-portable.zip   ttwriter.exe + wordcraft-cli.exe + licences

  The interface languages (31 catalogs) and the agent skills (13) are compiled
  into the binaries (include_str!), so the zip needs no data files.

  GNU builds don't take +crt-static (the MinGW runtime is linked dynamically
  and ships inside the binaries); libgcc/stdc++ are static in rustc's GNU
  target by default. The .winres icon/VERSIONINFO embedding works the same.

.EXAMPLE
  pwsh packaging/windows/package-gnu.ps1
  pwsh packaging/windows/package-gnu.ps1 -SkipBuild    # reuse target-gnu\...\release
#>
param(
  [switch] $SkipBuild
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

function Invoke-Native([string] $What, [scriptblock] $Block) {
  Write-Output "==> $What"
  & $Block
  if ($LASTEXITCODE -ne 0) { throw "$What failed with exit code $LASTEXITCODE" }
}

# The version lives in one place: [workspace.package] version in the root Cargo.toml (same rule
# as package.ps1: the binary reports it, the updater compares the release tag against it).
$Version = $env:WORDCRAFT_VERSION
$CargoVersion = $null
$inPkg = $false
foreach ($line in Get-Content (Join-Path $Root 'Cargo.toml')) {
  if ($line -match '^\s*\[') { $inPkg = ($line.Trim() -eq '[workspace.package]'); continue }
  if ($inPkg -and $line -match '^\s*version\s*=\s*"([^"]+)"') { $CargoVersion = $Matches[1]; break }
}
if (-not $CargoVersion) { throw 'could not read [workspace.package] version from Cargo.toml' }
if (-not $Version) { $Version = $CargoVersion }
elseif ($Version -ne $CargoVersion) {
  throw "WORDCRAFT_VERSION is $Version but [workspace.package] version in Cargo.toml is $CargoVersion.`n" +
    "The binary would report the Cargo.toml version, and the updater compares it with the release tag.`n" +
    "Set the Cargo.toml version to the release version (and commit it) before packaging."
}

$Target = 'x86_64-pc-windows-gnu'
$Toolchain = if ($env:WORDCRAFT_GNU_TOOLCHAIN) { $env:WORDCRAFT_GNU_TOOLCHAIN } else { "+stable-$Target" }
$Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist\release-gnu' }
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target-gnu' }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

if (-not $env:WORDCRAFT_BUILD_SHA) { $env:WORDCRAFT_BUILD_SHA = (git -C $Root rev-parse HEAD 2>$null) }
if (-not $env:WORDCRAFT_BUILD_DATE) { $env:WORDCRAFT_BUILD_DATE = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd') }

Write-Output "WordCraft $Version for Windows x64 (GNU toolchain)"

# Fonts: same policy as package.ps1 — no embedded CJK fonts by default.
Remove-Item Env:\CRAFT_FONTS_DIR -ErrorAction SilentlyContinue
Write-Output '==> fonts: not embedded (default; every Chinese Windows has Song/YaHei)'

if (-not $SkipBuild) {
  # Fail the build (rather than warn) if the icon/VERSIONINFO can't be embedded.
  $env:WORDCRAFT_REQUIRE_WINRES = '1'
  # cargo may not be on PATH under a bare PowerShell (the dev shell sets it); fall back to
  # the default install location.
  $Cargo = (Get-Command cargo -ErrorAction SilentlyContinue).Source
  if (-not $Cargo) { $Cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe' }
  if (-not (Test-Path $Cargo)) { throw "cargo not found (PATH or $Cargo)" }
  Invoke-Native "cargo build ($Target)" {
    & $Cargo $Toolchain build --release --locked -p wordcraft -p wordcraft-cli --target $Target
  }
}

$Bin = Join-Path $TargetDir "$Target\release"
foreach ($exe in 'wordcraft.exe', 'wordcraft-cli.exe') {
  if (-not (Test-Path (Join-Path $Bin $exe))) { throw "missing $exe in $Bin — build first (drop -SkipBuild)" }
}

# PE headers: machine must be x64; the app is a GUI binary, the CLI a console one.
function Get-PeHeader([string] $Path) {
  $bytes = [System.IO.File]::ReadAllBytes($Path)
  $pe = [BitConverter]::ToInt32($bytes, 0x3C)
  return @{ Machine = [BitConverter]::ToUInt16($bytes, $pe + 4); Subsystem = [BitConverter]::ToUInt16($bytes, $pe + 0x5C) }
}
foreach ($check in @(@('wordcraft.exe', 2), @('wordcraft-cli.exe', 3))) {
  $h = Get-PeHeader (Join-Path $Bin $check[0])
  if ($h.Machine -ne 0x8664) { throw "$($check[0]) is for machine 0x$('{0:X}' -f $h.Machine), expected 0x8664 (x64)" }
  if ($h.Subsystem -ne $check[1]) { throw "$($check[0]) has PE subsystem $($h.Subsystem), expected $($check[1])" }
  Write-Output "ok $($check[0]): x64 (GNU), PE subsystem $($h.Subsystem)"
}

# ---- portable zip ------------------------------------------------------------------------------
# The stage folder's name becomes the zip's top folder: the versioned portable name (same
# convention as package.ps1). The updater's find_install_root looks for the exe under it.
$Stage = Join-Path $TargetDir "windows-package\wordcraft-$Version-windows-x64-portable"
Remove-Item -Recurse -Force $Stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Stage | Out-Null
Copy-Item (Join-Path $Bin 'wordcraft-cli.exe') $Stage
# The shipped app is ttwriter.exe (蜻蜓文档 TTWriter); the cargo target keeps the upstream name.
Copy-Item (Join-Path $Bin 'wordcraft.exe') (Join-Path $Stage 'ttwriter.exe')

# GNU builds link the MinGW runtime into the binaries themselves; nothing extra to bundle.
# Licences: same set as package.ps1's portable zip.
foreach ($f in 'README.md', 'README.zh-CN.md', 'LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'ATTRIBUTION.md') {
  $p = Join-Path $Root $f
  if (Test-Path $p) { Copy-Item $p $Stage }
}

# A short pointer for zip users: what this is, how to start it, how to update.
$zhNote = @"
蜻蜓文档 TTWriter $Version（便携版，GNU 构建）

- 双击 ttwriter.exe 启动；wordcraft-cli.exe 是命令行工具（转换、PDF 工具、更新）。
- 免安装：解压即用。界面语言和 13 个文档技能都已在程序里，无需另配。
- 检查更新：wordcraft-cli update --check；下载并安装：wordcraft-cli update --install。
- 许可与第三方素材见 LICENSE-*、NOTICE、ATTRIBUTION.md。
"@
[System.IO.File]::WriteAllText((Join-Path $Stage '使用说明.txt'), $zhNote, [System.Text.Encoding]::UTF8)

$Zip = Join-Path $Dist "wordcraft-$Version-windows-x64-portable.zip"
Remove-Item -Force $Zip -ErrorAction SilentlyContinue
Compress-Archive -Path $Stage -DestinationPath $Zip

# Smoke: the CLI must run and report this build's version from this machine.
Invoke-Native 'wordcraft-cli --version (from the staged zip copy)' {
  & (Join-Path $Stage 'wordcraft-cli.exe') --version
}

# Update manifest (latest.json): same generator as the MSVC flow, so both channels stay in sync.
# -Dist matters: the generator scans it for the assets to record (this flow's zip, not the
# MSVC dist's).
& (Join-Path $PSScriptRoot 'manifest.ps1') -Version $Version -Dist $Dist

Get-Item $Zip | Format-Table Name, Length
Write-Output "sha256: $((Get-FileHash $Zip -Algorithm SHA256).Hash.ToLower())"

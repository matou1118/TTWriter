<#
.SYNOPSIS
  Build the web app (trunk) and stage the PWA assets next to it.

.DESCRIPTION
  One command for a complete, servable dist/web:
    1. trunk build --release            (page + JS + wasm; needs wasm32 target,
                                         wasm-bindgen 0.2.x, trunk 0.21.x)
    2. copy the PWA files into place    (manifest, sw.js, icons - plain copies;
                                         trunk's copy-file pipeline writes 1-byte
                                         stubs on 0.21.14, so this stays manual here
                                         and in pages.yml's assemble step - keep the
                                         two lists in sync)

  The local toolchain on this machine: MSVC links nothing, so run under
  RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-gnu; trunk and wasm-bindgen are
  prebuilt binaries in target-gnu\trunk-bin (see README notes in repo docs).

.EXAMPLE
  powershell -File build-web.ps1
#>
param(
  [switch] $SkipBuild
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$App = Join-Path $Root 'apps\wordcraft-web'
$Dist = Join-Path $Root 'dist\web'

# Toolchain locations (prebuilt; PATH usually does not carry them in a bare shell).
$Trunk = Join-Path $Root 'target-gnu\trunk-bin\x\trunk.exe'
$WbgDir = Join-Path $Root 'target-gnu\trunk-bin\wbg\wasm-bindgen-0.2.129-x86_64-pc-windows-msvc'
if (-not (Test-Path $Trunk)) { throw "trunk not found: $Trunk (see the script header)" }

if (-not $SkipBuild) {
  $env:RUSTUP_TOOLCHAIN = 'stable-x86_64-pc-windows-gnu'
  $env:PATH = "$WbgDir;$env:USERPROFILE\.cargo\bin;$env:PATH"
  $env:CARGO_TARGET_DIR = 'target-gnu'
  Push-Location $App
  try {
    # PS5.1 把 native stderr（cargo 的 rustc 警告）转成 ErrorRecord；EAP=Stop 会中断
    # 脚本。构建成败只看退出码——调用前后放宽 EAP。
    $eap = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
      & $Trunk build --release 2>&1 | Out-Host
    } finally {
      $ErrorActionPreference = $eap
    }
    if ($LASTEXITCODE -ne 0) { throw "trunk build failed (exit $LASTEXITCODE)" }
  } finally { Pop-Location }
}

# ---- PWA assets (same list as pages.yml's assemble step) ------------------------
$icons = Join-Path $Root 'assets\app-icon'
New-Item -ItemType Directory -Force -Path $Dist | Out-Null
Copy-Item (Join-Path $App 'manifest.json') $Dist -Force
Copy-Item (Join-Path $App 'sw.js') $Dist -Force
Copy-Item (Join-Path $icons 'hicolor\512x512\apps\ai.storyteller.wordcraft.png') (Join-Path $Dist 'wordcraft-icon-512.png') -Force
Copy-Item (Join-Path $icons 'hicolor\256x256\apps\ai.storyteller.wordcraft.png') (Join-Path $Dist 'wordcraft-icon-256.png') -Force
Copy-Item (Join-Path $icons 'hicolor\128x128\apps\ai.storyteller.wordcraft.png') (Join-Path $Dist 'wordcraft-icon-128.png') -Force
Copy-Item (Join-Path $icons 'wordcraft.svg') $Dist -Force
# 上一次构建若写出了坏图标（1 字节 copy-file 残留），这里覆盖掉了。

Get-ChildItem $Dist | ForEach-Object { Write-Output ("{0}  {1:N0} bytes" -f $_.Name, $_.Length) }
Write-Output "serve locally:  python -m http.server 8774 --directory $Dist"

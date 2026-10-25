<#
.SYNOPSIS
  Shard the trunk-built _bg.wasm into <=18MB pieces for jsdelivr gh mirroring.

.DESCRIPTION
  jsdelivr's gh mirror caps single files at ~20MB; our wasm is ~55MB. This splits
  it into 18MB shards written to cdn-shards/ (committed, mirrored by jsdelivr) and
  dist/web/shards/ (local fallback served from Pages), plus a manifest.json the
  index.html fetch-intercept uses to reassemble before wasm-bindgen inits.

  Run after build-web.ps1 (which produces dist/web/*_bg.wasm).

.EXAMPLE
  powershell -File shard-wasm.ps1
#>
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$Dist = Join-Path $Root 'dist\web'
$Cdn  = Join-Path $Root 'cdn-shards'

$wasm = Get-ChildItem (Join-Path $Dist '*_bg.wasm') | Select-Object -First 1
if (-not $wasm) { throw "no *_bg.wasm in $Dist — run build-web.ps1 first" }

$bytes = [System.IO.File]::ReadAllBytes($wasm.FullName)
$size  = $bytes.Length
$shard = 18 * 1048576  # 18MB, safely under jsdelivr's ~20MB cap
$n     = [int][Math]::Ceiling($size / $shard)

New-Item -ItemType Directory -Force -Path $Cdn | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $Dist 'shards') | Out-Null
Get-ChildItem $Cdn -File | Remove-Item -Force
Get-ChildItem (Join-Path $Dist 'shards') -File -ErrorAction SilentlyContinue | Remove-Item -Force

$files = @()
for ($i = 0; $i -lt $n; $i++) {
  $start = $i * $shard
  $len   = [Math]::Min($shard, $size - $start)
  $name  = "$i.bin"
  $buf   = [byte[]]::new($len)
  [Array]::Copy($bytes, $start, $buf, 0, $len)
  $cdnPath = Join-Path $Cdn $name
  [System.IO.File]::WriteAllBytes($cdnPath, $buf)
  Copy-Item $cdnPath (Join-Path $Dist "shards\$name") -Force
  $files += $name
}

$manifest = [ordered]@{ size = $size; shards = $files; source = $wasm.Name }
$mj = $manifest | ConvertTo-Json -Compress
[System.IO.File]::WriteAllText((Join-Path $Cdn 'manifest.json'), $mj, [System.Text.UTF8Encoding]::new($false))
[System.IO.File]::WriteAllText((Join-Path $Dist 'shards\manifest.json'), $mj, [System.Text.UTF8Encoding]::new($false))

Write-Output ("wasm {0:N0} bytes -> {1} shards of <=18MB in cdn-shards/ (+ dist/web/shards/ fallback)" -f $size, $n)
Get-ChildItem $Cdn -File | ForEach-Object { Write-Output ("  {0}  {1:N0}" -f $_.Name, $_.Length) }

<#
.SYNOPSIS
  Write the update manifest (latest.json) for a release.

.DESCRIPTION
  The manifest is what an updater reads to learn that a new version exists: a few hundred bytes
  with the version, the download URLs per channel and the SHA-256 of every asset. It lives in
  the repository, so the updater reads it from raw hosts — Gitee first (a static file, no API
  quota, the reliable path inside China), then AtomGit's contents API, then GitHub's raw host —
  and checking for updates costs no bandwidth worth measuring. Only the download itself does,
  and that points at the release assets on GitHub (global) and AtomGit (China), which host them
  for free (Gitee's 100 MB attachment cap cannot hold the installer, so it carries the
  manifest only).

  The manifest is written to the site directory as `latest.json`, and to the dist directory
  next to the artifacts, so a release can upload it as a release asset too. Publishing it to
  the site is a `git add site/latest.json && git commit` away; the URLs in it point at the
  release assets, which are uploaded the usual way.

.EXAMPLE
  pwsh packaging/windows/manifest.ps1 -Version 0.9.0-zh.2
#>
param(
  # The release version, e.g. 0.9.0-zh.2. Defaults to the tag the checkout is on.
  [string] $Version,
  [string] $Dist,
  [string] $SiteDir,
  [string] $GitHubRepo = 'matou1118/ttwriter',
  [string] $AtomGitRepo = 'matou118/ttwriter',
  [string] $Notes = ''
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

if (-not $Version) {
  $tag = (git -C $Root describe --tags --abbrev=0 2>$null)
  if ($tag) { $Version = $tag.TrimStart('v') }
}
if (-not $Version) { throw 'no version: pass -Version or tag the checkout' }
$Tag = "v$Version"
if (-not $Dist) { $Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist\release' } }
if (-not $SiteDir) { $SiteDir = Join-Path $Root 'site' }

$assets = Get-ChildItem $Dist -File -ErrorAction SilentlyContinue |
  Where-Object { $_.Extension -in '.msi', '.zip' } |
  Where-Object { $_.Name -notlike '*.wixpdb' }
if (-not $assets) { throw "no release assets in $Dist" }

$entries = foreach ($a in $assets) {
  $sha = (Get-FileHash $a.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
  foreach ($channel in @(
      @{ name = 'cn'; base = "https://atomgit.com/$AtomGitRepo/releases/download/$Tag" },
      @{ name = 'global'; base = "https://github.com/$GitHubRepo/releases/download/$Tag" })) {
    [ordered]@{
      name    = $a.Name
      channel = $channel.name
      url     = "$($channel.base)/$($a.Name)"
      sha256  = $sha
      size    = $a.Length
    }
  }
}

$manifest = [ordered]@{
  version   = $Version
  tag       = $Tag
  published = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
  notes     = $Notes
  assets    = @($entries)
}
$json = $manifest | ConvertTo-Json -Depth 4

$siteFile = Join-Path $SiteDir 'latest.json'
$distFile = Join-Path $Dist 'latest.json'
foreach ($f in @($siteFile, $distFile)) {
  New-Item -ItemType Directory -Force -Path (Split-Path $f) | Out-Null
  # UTF-8 without a BOM: the updater reads it as JSON.
  [System.IO.File]::WriteAllText($f, $json, (New-Object System.Text.UTF8Encoding($false)))
}

Write-Output "==> wrote $siteFile and $distFile"
$manifest.assets | ForEach-Object { Write-Output ("    {0,-46} {1,-7} {2,10:N1} MB  {3}" -f $_.name, $_.channel, ($_.size / 1MB), $_.sha256.Substring(0, 16)) }

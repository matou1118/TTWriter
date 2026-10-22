# 构建（Windows）
#
# GNU 工具链 + MinGW 的完整配方。test-all.ps1 也用同一套环境变量。
# 用法：  powershell -ExecutionPolicy Bypass -File build.ps1
#
# 中文字体是可选构建输入（不进仓库）：有 craft-fonts\fonts\manifest.txt 就带上，
# 出厂 exe 才会在没装中文字体的机器上兜底。没有就编译出「不自带中文」的版本。

param([switch]$SkipFonts)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot

$toolchain = "$env:USERPROFILE\.rustup\toolchains\stable-x86_64-pc-windows-gnu"
$selfCont  = "$toolchain\lib\rustlib\x86_64-pc-windows-gnu\bin\self-contained"
$mingw     = 'C:\ProgramData\mingw64\mingw64\bin'
$env:PATH = "$mingw;$selfCont;$env:USERPROFILE\.cargo\bin;$env:PATH"
$env:CARGO_TARGET_DIR = "$root\target-gnu"

$craft = "$root\craft-fonts"
if (-not $SkipFonts -and (Test-Path "$craft\fonts\manifest.txt")) {
    $env:CRAFT_FONTS_DIR = $craft
    Write-Host "中文字体：已嵌入（$craft）" -ForegroundColor Cyan
} else {
    Remove-Item Env:\CRAFT_FONTS_DIR -ErrorAction SilentlyContinue
    Write-Host "中文字体：未嵌入（缺 craft-fonts\fonts\manifest.txt）" -ForegroundColor Yellow
}

& cargo +stable-x86_64-pc-windows-gnu build --release -p wordcraft -p wordcraft-cli
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$exe = "$root\target-gnu\release\wordcraft.exe"
if (Test-Path $exe) {
    Write-Host ("完成：{0}  {1:N1} MB" -f $exe, ((Get-Item $exe).Length / 1MB)) -ForegroundColor Green
}

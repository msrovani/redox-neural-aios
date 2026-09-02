# Verifica stack Redox Neural AIOS no host (Fase 0/1 aceite parcial).
$ErrorActionPreference = "Stop"
$AiosRoot = Split-Path -Parent $PSScriptRoot
Set-Location $AiosRoot

Write-Host "=== verify-stack: cargo test ===" -ForegroundColor Cyan
cargo test --workspace
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n=== verify-stack: scheme memory (TCP) ===" -ForegroundColor Cyan
$sgdbPath = Join-Path $env:TEMP "redox-sgdb-verify"
New-Item -ItemType Directory -Force -Path $sgdbPath | Out-Null
$env:REDOX_SGDB_PATH = $sgdbPath
$env:REDOX_MEMORY_BACKEND = "tcp"

$sgdbdJob = Start-Job -ScriptBlock {
    param($root, $path)
    Set-Location $root
    $env:REDOX_SGDB_PATH = $path
    cargo run -q -p sgdbd --bin sgdbd 2>&1
} -ArgumentList $AiosRoot, $sgdbPath

Start-Sleep -Seconds 2
cargo run -q -p sgdbd --bin memory -- remember "verify-stack ok" --scope boot
if ($LASTEXITCODE -ne 0) {
    Stop-Job $sgdbdJob -ErrorAction SilentlyContinue
    Remove-Job $sgdbdJob -Force -ErrorAction SilentlyContinue
    exit 1
}
cargo run -q -p sgdbd --bin memory -- recall "verify" --scope boot
$memOk = $LASTEXITCODE -eq 0
Stop-Job $sgdbdJob -ErrorAction SilentlyContinue
Remove-Job $sgdbdJob -Force -ErrorAction SilentlyContinue

if (-not $memOk) { exit 1 }

Write-Host "`n=== verify-stack: scheme memory (file bridge) ===" -ForegroundColor Cyan
$schemeRoot = Join-Path $env:TEMP "scheme\memory"
Remove-Item -Recurse -Force $schemeRoot -ErrorAction SilentlyContinue
$env:REDOX_MEMORY_SCHEME_ROOT = $schemeRoot
$env:REDOX_SGDB_PATH = Join-Path $env:TEMP "redox-sgdb-scheme"
New-Item -ItemType Directory -Force -Path $env:REDOX_SGDB_PATH | Out-Null

$sgdbdJob2 = Start-Job -ScriptBlock {
    param($root, $path, $scheme)
    Set-Location $root
    $env:REDOX_SGDB_PATH = $path
    $env:REDOX_MEMORY_SCHEME_ROOT = $scheme
    cargo run -q -p sgdbd --bin sgdbd 2>&1
} -ArgumentList $AiosRoot, $env:REDOX_SGDB_PATH, $schemeRoot

Start-Sleep -Seconds 2
$env:REDOX_MEMORY_BACKEND = "scheme"
cargo run -q -p sgdbd --bin memory -- remember "scheme bridge ok" --scope boot
$schemeOk = $LASTEXITCODE -eq 0
Stop-Job $sgdbdJob2 -ErrorAction SilentlyContinue
Remove-Job $sgdbdJob2 -Force -ErrorAction SilentlyContinue

if (-not $schemeOk) { exit 1 }

Write-Host "`nverify-stack: OK" -ForegroundColor Green

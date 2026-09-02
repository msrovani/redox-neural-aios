# Demo QEMU E2E — Redox Neural AIOS (Fase 0 aceite + smoke guest + escada).
param(
    [switch]$BuildOnly,
    [switch]$SkipBuild,
    [switch]$FullLadder,
    [string]$RedoxRoot = ""
)
$ErrorActionPreference = "Stop"
$AiosRoot = Split-Path -Parent $PSScriptRoot
Set-Location $AiosRoot

if (-not $RedoxRoot) {
    $RedoxRoot = Join-Path (Split-Path -Parent $AiosRoot) "redox"
}

Write-Host "=== Demo QEMU Redox Neural AIOS ===" -ForegroundColor Cyan
Write-Host "Redox root: $RedoxRoot" -ForegroundColor DarkGray

if (-not $SkipBuild) {
    Write-Host "`n[1/3] Build ISO (WSL)..." -ForegroundColor Yellow
    & (Join-Path $AiosRoot "tools\build-wsl.ps1") -Target "aios-minimal" -RedoxRoot $RedoxRoot
    if ($LASTEXITCODE -ne 0) { throw "build aios-minimal falhou" }
}

if ($BuildOnly) {
    Write-Host "BuildOnly — ISO pronta." -ForegroundColor Green
    exit 0
}

$iso = Join-Path $RedoxRoot "build\x86_64\desktop\harddrive.img"
if (-not (Test-Path $iso)) {
    Write-Host "ISO não encontrada: $iso" -ForegroundColor Yellow
    Write-Host "Execute com -SkipBuild após build manual, ou verifique path." -ForegroundColor DarkGray
}

Write-Host "`n[2/3] Instruções QEMU" -ForegroundColor Yellow
Write-Host @"
No WSL/Linux (a partir de $RedoxRoot):
  make qemu
  # ou:
  qemu-system-x86_64 -serial stdio -hda build/x86_64/desktop/harddrive.img

No guest (login), copie/execute:
  sh /usr/share/aios/qemu-guest-check.sh
"@ -ForegroundColor DarkGray

Write-Host "`n[3/3] Host verify (pre-QEMU)" -ForegroundColor Yellow
& (Join-Path $AiosRoot "tools\verify-stack.ps1")
if ($LASTEXITCODE -ne 0) { throw "verify-stack falhou" }

if ($FullLadder) {
    Write-Host "`n[extra] Host escada completa (baseline)" -ForegroundColor Yellow
    & (Join-Path $AiosRoot "tools\demo-ladder.ps1") -WithNet -FullLadder
    if ($LASTEXITCODE -ne 0) { throw "demo-ladder falhou" }
}

Write-Host "`n=== Demo QEMU preparada ===" -ForegroundColor Green
Write-Host "Aceite guest: sh /usr/share/aios/qemu-guest-check.sh"
Write-Host "Aceite gravável: memory URI + hermes /factory + /evolve + /promote list"

# Demo escada cognitiva ADR-010 (Onda 7i) — host.
# Exemplo de usabilidade: mesma intent 3× → SKILL.md; providers HTTP opcionais.
param(
    [string]$Intent = "qual a temperatura em sp",
    [switch]$WithNet
)

$ErrorActionPreference = "Stop"
$AiosRoot = Split-Path -Parent $PSScriptRoot
Set-Location $AiosRoot

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo não encontrado no PATH"
}

$hermesOk = $false
try {
    $c = New-Object System.Net.Sockets.TcpClient
    $c.Connect("127.0.0.1", 7742)
    $c.Close()
    $hermesOk = $true
} catch {}

if (-not $hermesOk) {
    Write-Host "hermesd (7742) offline — suba a stack: .\tools\start-stack.ps1 ou .\tools\demo-e2e.ps1 -KeepStack" -ForegroundColor Yellow
    exit 1
}

if ($WithNet) {
    $env:REDOX_TOOLS_NET = "1"
    $env:REDOX_TOOLS_PROVIDERS = "open_meteo"
    Write-Host "Providers: REDOX_TOOLS_NET=1 REDOX_TOOLS_PROVIDERS=open_meteo" -ForegroundColor DarkGray
}

Write-Host "=== Demo escada cognitiva ===" -ForegroundColor Cyan
Write-Host "Intent: `"$Intent`"" -ForegroundColor DarkGray

for ($i = 1; $i -le 3; $i++) {
    Write-Host "`n[$i/3] efêmera (degrau 0)..." -ForegroundColor Yellow
    cargo run -q -p hermesd --bin hermes -- $Intent
}

Write-Host "`n[skill] /evolve — skills geradas" -ForegroundColor Yellow
cargo run -q -p hermesd --bin hermes -- "/evolve"

Write-Host "`n[4] workflow SKILL.md (4ª execução)" -ForegroundColor Yellow
cargo run -q -p hermesd --bin hermes -- $Intent

Write-Host "`n[tools] registry" -ForegroundColor Yellow
cargo run -q -p hermesd --bin hermes -- "/tools"

Write-Host "`n=== Demo escada concluída ===" -ForegroundColor Green
Write-Host "Próximo: runs maduros → WASM; /promote <skill> approve → app wasmi (HITL)."

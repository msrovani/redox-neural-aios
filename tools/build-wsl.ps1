# Build Redox Neural AIOS via WSL (quando make não está no Windows).
param(
    [string]$Target = "aios-minimal",
    [string]$RedoxRoot = ""
)

$ErrorActionPreference = "Stop"
if (-not $RedoxRoot) {
    $RedoxRoot = Join-Path (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)) "redox"
}

$wsl = Get-Command wsl -ErrorAction SilentlyContinue
if (-not $wsl) {
    Write-Error @"
WSL não encontrado. Para build Redox no Windows:
  1. Instale WSL2 + Ubuntu
  2. Siga https://doc.redox-os.org/book/podman-build.html
  3. Dentro do WSL: cd /mnt/c/DEV/redox && make $Target

Ou use Linux/macOS nativo com make instalado.
"@
}

$AiosRoot = Split-Path -Parent $PSScriptRoot
Write-Host "Aplicando overlay..."
& (Join-Path $AiosRoot "tools\bootstrap.ps1") -RedoxRoot $RedoxRoot

$wslRedox = $RedoxRoot -replace '\\', '/' -replace '^C:', '/mnt/c' -replace '^c:', '/mnt/c'
Write-Host "Build WSL: make $Target em $wslRedox"
wsl bash -lc "cd '$wslRedox' && make $Target"

# Sincroniza upstream Redox e reaplica overlay AIOS.

param(
    [string]$RedoxRoot = (Join-Path (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)) "redox"),
    [string]$Branch = "master"
)

$ErrorActionPreference = "Stop"
$AiosRoot = Split-Path -Parent $PSScriptRoot

if (-not (Test-Path $RedoxRoot)) {
    Write-Error "Redox root nao encontrado: $RedoxRoot. Execute tools/bootstrap.ps1 -Clone primeiro."
}

Write-Host "Sync upstream Redox ($Branch)..."
Push-Location $RedoxRoot
try {
    git fetch origin
    git checkout $Branch
    git pull origin $Branch
} finally {
    Pop-Location
}

Write-Host "Reaplicando overlay AIOS..."
& (Join-Path $AiosRoot "tools\apply-to-redox.ps1") -RedoxRoot $RedoxRoot

Write-Host "Sync concluido."

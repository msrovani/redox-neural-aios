# Redox AIOS — bootstrap do fork
# Clona ou vincula o upstream Redox e aplica o overlay AIOS.

param(
    [string]$RedoxRoot = "",
    [switch]$Clone
)

$ErrorActionPreference = "Stop"
$AiosRoot = Split-Path -Parent $PSScriptRoot

if (-not $RedoxRoot) {
    $RedoxRoot = Join-Path (Split-Path -Parent $AiosRoot) "redox"
}

Write-Host "Redox AIOS bootstrap"
Write-Host "  AIOS root:  $AiosRoot"
Write-Host "  Redox root: $RedoxRoot"

if ($Clone -and -not (Test-Path $RedoxRoot)) {
    Write-Host "Clonando upstream Redox..."
    git clone https://gitlab.redox-os.org/redox-os/redox.git $RedoxRoot
}

if (-not (Test-Path $RedoxRoot)) {
    Write-Error "Redox root nao encontrado: $RedoxRoot. Use -Clone ou defina -RedoxRoot."
}

& (Join-Path $AiosRoot "tools\apply-to-redox.ps1") -RedoxRoot $RedoxRoot

Write-Host ""
Write-Host "Bootstrap concluido."
Write-Host "Build: cd $RedoxRoot && make aios-minimal"

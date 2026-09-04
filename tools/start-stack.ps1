# Sobe a stack cognitiva Redox Neural AIOS no host (6 daemons).
param(
    [string]$SgdbPath = ""
)

$ErrorActionPreference = "Stop"
$AiosRoot = Split-Path -Parent $PSScriptRoot
Set-Location $AiosRoot

if (-not $SgdbPath) {
    $SgdbPath = Join-Path $env:TEMP "redox-sgdb"
}
New-Item -ItemType Directory -Force -Path $SgdbPath | Out-Null
$env:REDOX_SGDB_PATH = $SgdbPath
$env:REDOX_AIOS_SCHEME_ROOT = Join-Path $env:TEMP "scheme\aios"
New-Item -ItemType Directory -Force -Path $env:REDOX_AIOS_SCHEME_ROOT | Out-Null

$jobs = @(
    @{ Name = "eventd";  Bin = "eventd" },
    @{ Name = "sgdbd";   Bin = "sgdbd" },
    @{ Name = "cortexd"; Bin = "cortexd" },
    @{ Name = "hermesd"; Bin = "hermesd" },
    @{ Name = "voiced";  Bin = "voiced" },
    @{ Name = "jarbasd"; Bin = "jarbasd" }
)
# mcpd é stdio (Cursor); para TCP no host: cargo run -p mcpd -- --tcp

Write-Host "Iniciando stack AIOS (SGDB=$SgdbPath)..." -ForegroundColor Cyan
$started = @()
foreach ($j in $jobs) {
    $job = Start-Job -Name $j.Name -ScriptBlock {
        param($root, $path, $bin)
        Set-Location $root
        $env:REDOX_SGDB_PATH = $path
        $env:REDOX_AIOS_SCHEME_ROOT = Join-Path $env:TEMP "scheme\aios"
        cargo run -q -p $bin --bin $bin 2>&1
    } -ArgumentList $AiosRoot, $SgdbPath, $j.Bin
    $started += $job
    Start-Sleep -Milliseconds 400
}

Start-Sleep -Seconds 3
Write-Host "`nJobs ativos:" -ForegroundColor Green
Get-Job | Format-Table Id, Name, State

Write-Host @"

Stack em background. Testar:
  cargo run -p hermesd --bin hermes -- intent "que horas são"
  cargo run -p jarbasd --bin jarbas -- chat "olá jarbas"

Parar: Get-Job | Stop-Job; Get-Job | Remove-Job
"@

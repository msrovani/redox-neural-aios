# Demo Jarvis E2E no host (Fase 6 — sem MIC nativo).
param(
    [switch]$KeepStack
)

$ErrorActionPreference = "Continue"
$AiosRoot = Split-Path -Parent $PSScriptRoot
Set-Location $AiosRoot

$SgdbPath = Join-Path $env:TEMP "redox-sgdb-demo"
Remove-Item -Recurse -Force $SgdbPath -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $SgdbPath | Out-Null
$env:REDOX_SGDB_PATH = $SgdbPath
$env:REDOX_JARBAS_BOOT_GREET = "0"
$env:REDOX_AIOS_SCHEME_ROOT = Join-Path $env:TEMP "scheme\aios"
New-Item -ItemType Directory -Force -Path $env:REDOX_AIOS_SCHEME_ROOT | Out-Null

function Wait-TcpPort($port, $timeoutSec = 60) {
    $deadline = (Get-Date).AddSeconds($timeoutSec)
    while ((Get-Date) -lt $deadline) {
        try {
            $c = New-Object System.Net.Sockets.TcpClient
            $c.Connect("127.0.0.1", $port)
            $c.Close()
            return $true
        } catch {
            Start-Sleep -Milliseconds 500
        }
    }
    return $false
}

function Start-Daemon($name) {
    Start-Job -Name $name -ScriptBlock {
        param($root, $path, $bin)
        Set-Location $root
        $env:REDOX_SGDB_PATH = $path
        $env:REDOX_JARBAS_BOOT_GREET = "0"
        $env:REDOX_AIOS_SCHEME_ROOT = Join-Path $env:TEMP "scheme\aios"
        cargo run -q -p $bin --bin $bin 2>&1
    } -ArgumentList $AiosRoot, $SgdbPath, $name | Out-Null
}

Get-Job | Stop-Job -ErrorAction SilentlyContinue
Get-Job | Remove-Job -Force -ErrorAction SilentlyContinue

Write-Host "=== Demo E2E Redox Neural AIOS ===" -ForegroundColor Cyan
Write-Host "Compilando daemons..."
cargo build -q -p eventd -p sgdbd -p cortexd -p hermesd -p voiced -p jarbasd *> $null
if ($LASTEXITCODE -ne 0) { throw "cargo build falhou" }

@("eventd", "sgdbd", "cortexd", "hermesd", "voiced", "jarbasd") | ForEach-Object {
    Write-Host "  subindo $_..."
    Start-Daemon $_
    Start-Sleep -Milliseconds 300
}

Write-Host "Aguardando portas..."
$ports = @{ sgdbd = 7741; hermesd = 7742; cortexd = 7743; voiced = 7744; jarbasd = 7745 }
foreach ($entry in $ports.GetEnumerator()) {
    if (-not (Wait-TcpPort $entry.Value 90)) {
        Write-Host "Jobs:" ; Get-Job | Format-Table
        throw "Timeout: $($entry.Key) porta $($entry.Value)"
    }
    Write-Host "  OK $($entry.Key):$($entry.Value)" -ForegroundColor DarkGray
}

Write-Host "`n[1/5] memory remember/recall" -ForegroundColor Yellow
cargo run -q -p sgdbd --bin memory -- remember "demo e2e boot ok" --scope boot
if ($LASTEXITCODE -ne 0) { throw "memory remember falhou" }
cargo run -q -p sgdbd --bin memory -- recall "demo" --scope boot

Write-Host "`n[2/5] hermes intent (time)" -ForegroundColor Yellow
cargo run -q -p hermesd --bin hermes -- "que horas são"

Write-Host "`n[3/5] hermes HITL block" -ForegroundColor Yellow
cargo run -q -p hermesd --bin hermes -- "rm -rf /"

Write-Host "`n[4/5] voiced utterance" -ForegroundColor Yellow
cargo run -q -p voiced --bin voice -- utterance "jarbas, /time"

Write-Host "`n[5/5] jarbas chat" -ForegroundColor Yellow
cargo run -q -p jarbasd --bin jarbas -- chat "/echo demo e2e ok"

Write-Host "`n=== Demo E2E concluída ===" -ForegroundColor Green

if (-not $KeepStack) {
    Get-Job | Stop-Job -ErrorAction SilentlyContinue
    Get-Job | Remove-Job -Force -ErrorAction SilentlyContinue
    Write-Host "Stack encerrada (-KeepStack para manter jobs)."
} else {
    Write-Host "Stack mantida. Parar: Get-Job | Stop-Job; Get-Job | Remove-Job"
}

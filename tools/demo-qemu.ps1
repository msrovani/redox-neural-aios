# Demo QEMU E2E gravavel - Redox Neural AIOS
# Host: gera evidencia em docs/memory/evidence/
# Guest: apos make qemu, rode sh /usr/share/aios/qemu-guest-check.sh
param(
    [switch]$BuildOnly,
    [switch]$SkipBuild,
    [switch]$HostOnly,
    [switch]$FullLadder,
    [switch]$SkipRecord,
    [switch]$FullCargoTest,
    [string]$RedoxRoot = "",
    [string]$EvidenceDir = ""
)

$ErrorActionPreference = "Continue"
$AiosRoot = Split-Path -Parent $PSScriptRoot
Set-Location $AiosRoot

if (-not $RedoxRoot) {
    $RedoxRoot = Join-Path (Split-Path -Parent $AiosRoot) "redox"
}
if (-not $EvidenceDir) {
    $EvidenceDir = Join-Path $AiosRoot "docs\memory\evidence"
}
New-Item -ItemType Directory -Force -Path $EvidenceDir | Out-Null

$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$evidencePath = Join-Path $EvidenceDir "qemu-e2e-$stamp.md"
$logPath = Join-Path $EvidenceDir "qemu-e2e-$stamp.log"
$script:LogLines = @()

function Write-Log {
    param([string]$Message, [string]$Color = "White")
    Write-Host $Message -ForegroundColor $Color
    $script:LogLines += $Message
}

function Test-WslAvailable {
    try {
        $p = Start-Process -FilePath "wsl.exe" -ArgumentList @("-e", "true") -Wait -PassThru -WindowStyle Hidden
        return ($null -ne $p -and $p.ExitCode -eq 0)
    } catch {
        return $false
    }
}

function Test-HermesUp {
    try {
        $c = New-Object System.Net.Sockets.TcpClient
        $c.Connect("127.0.0.1", 7742)
        $c.Close()
        return $true
    } catch {
        return $false
    }
}

$hasWsl = Test-WslAvailable
$useHostOnly = [bool]$HostOnly
if (-not $hasWsl) {
    $useHostOnly = $true
    Write-Log "WSL ausente - modo HostOnly (baseline gravavel sem guest)." "Yellow"
}

Write-Log "=== Demo QEMU Redox Neural AIOS (gravavel) ===" "Cyan"
Write-Log "Redox root: $RedoxRoot"
Write-Log "Evidence: $evidencePath"
Write-Log "WSL: $hasWsl | HostOnly: $useHostOnly"

$stepsOk = @{
    build_iso    = $false
    iso_present  = $false
    verify_stack = $false
    hermes_up    = $false
    caps         = $false
    ladder       = $false
    mcp          = $false
}

if (-not $useHostOnly -and -not $SkipBuild) {
    Write-Log "`n[1] Build ISO (WSL)..." "Yellow"
    & (Join-Path $AiosRoot "tools\build-wsl.ps1") -Target "aios-minimal" -RedoxRoot $RedoxRoot
    if ($LASTEXITCODE -ne 0) { throw "build aios-minimal falhou" }
    $stepsOk.build_iso = $true
} else {
    Write-Log "[1] Build ISO pulado (HostOnly/SkipBuild)" "DarkGray"
}

if ($BuildOnly) {
    Write-Log "BuildOnly - ISO pronta." "Green"
    exit 0
}

$iso = Join-Path $RedoxRoot "build\x86_64\desktop\harddrive.img"
$isoExists = Test-Path $iso
$stepsOk.iso_present = $isoExists
if ($isoExists) {
    Write-Log "ISO: $iso" "DarkGray"
} else {
    Write-Log "ISO nao encontrada (esperado sem WSL build): $iso" "DarkGray"
}

Write-Log "`n[2] Host verify" "Yellow"
if ($FullCargoTest) {
    Write-Log "FullCargoTest: verify-stack completo" "DarkGray"
    & (Join-Path $AiosRoot "tools\verify-stack.ps1")
    if ($LASTEXITCODE -ne 0) { throw "verify-stack falhou" }
} else {
    Write-Log "verify rapido (memory TCP) - use -FullCargoTest para suite completa" "DarkGray"
    cargo build -q -p sgdbd --bin sgdbd --bin memory 2>$null
    $sgdbPath = Join-Path $env:TEMP "redox-sgdb-qemu-demo"
    New-Item -ItemType Directory -Force -Path $sgdbPath | Out-Null
    $env:REDOX_SGDB_PATH = $sgdbPath
    $env:REDOX_MEMORY_BACKEND = "tcp"
    $job = Start-Job -ScriptBlock {
        param($root, $path)
        Set-Location $root
        $env:REDOX_SGDB_PATH = $path
        cargo run -q -p sgdbd --bin sgdbd 2>&1
    } -ArgumentList $AiosRoot, $sgdbPath
    $deadline = (Get-Date).AddSeconds(90)
    $up = $false
    while ((Get-Date) -lt $deadline) {
        try {
            $c = New-Object System.Net.Sockets.TcpClient
            $c.Connect("127.0.0.1", 7741)
            $c.Close()
            $up = $true
            break
        } catch {
            Start-Sleep -Milliseconds 500
        }
    }
    if (-not $up) {
        Stop-Job $job -ErrorAction SilentlyContinue
        Remove-Job $job -Force -ErrorAction SilentlyContinue
        throw "sgdbd :7741 timeout"
    }
    cargo run -q -p sgdbd --bin memory -- remember "qemu host evidence ok" --scope boot
    if ($LASTEXITCODE -ne 0) {
        Stop-Job $job -ErrorAction SilentlyContinue
        Remove-Job $job -Force -ErrorAction SilentlyContinue
        throw "memory remember falhou"
    }
    cargo run -q -p sgdbd --bin memory -- recall "qemu" --scope boot
    $memOk = ($LASTEXITCODE -eq 0)
    Stop-Job $job -ErrorAction SilentlyContinue
    Remove-Job $job -Force -ErrorAction SilentlyContinue
    if (-not $memOk) { throw "memory recall falhou" }
}
$stepsOk.verify_stack = $true

$startedStack = $false
if (-not (Test-HermesUp)) {
    Write-Log "`n[2b] Subindo stack (demo-e2e -KeepStack)..." "Yellow"
    & (Join-Path $AiosRoot "tools\demo-e2e.ps1") -KeepStack
    if ($LASTEXITCODE -ne 0) { throw "demo-e2e falhou" }
    $startedStack = $true
    Start-Sleep -Seconds 2
}
$stepsOk.hermes_up = Test-HermesUp

Write-Log "`n[3] CapGate (host)" "Yellow"
$capsList = (& cargo run -q -p hermesd --bin hermes -- "/caps list" 2>&1 | Out-String)
Write-Log $capsList.TrimEnd()
$capsProbe = (& cargo run -q -p hermesd --bin hermes -- "/caps probe" 2>&1 | Out-String)
Write-Log $capsProbe.TrimEnd()
$stepsOk.caps = (($capsList -match "OS CAPS") -or ($capsList -match "REDOX NS"))
if (-not $stepsOk.caps) { throw "caps list falhou" }

Write-Log "`n[4] Escada cognitiva (host inline)" "Yellow"
$ladderOk = $true
foreach ($intent in @("/time", "/factory", "/evolve", "/promote list")) {
    Write-Log "  hermes $intent" "DarkGray"
    $out = (& cargo run -q -p hermesd --bin hermes -- $intent 2>&1 | Out-String)
    Write-Log $out.TrimEnd()
    if ($LASTEXITCODE -ne 0) { $ladderOk = $false }
}
$stepsOk.ladder = $ladderOk
if (-not $ladderOk) { throw "escada inline falhou" }

Write-Log "`n[5] MCP initialize (stdio smoke)" "Yellow"
$initLine = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
$mcpOut = $initLine | & cargo run -q -p mcpd --bin mcpd 2>$null
Write-Log (($mcpOut | Out-String).TrimEnd())
$stepsOk.mcp = (($mcpOut | Out-String) -match "redox-aios")
if (-not $stepsOk.mcp) { throw "mcp initialize falhou" }

Write-Log "`n[6] Instrucoes guest QEMU" "Yellow"
Write-Log "Quando WSL/ISO estiver pronto:" "DarkGray"
Write-Log "  cd $RedoxRoot" "DarkGray"
Write-Log "  make qemu" "DarkGray"
Write-Log "  sh /usr/share/aios/qemu-guest-check.sh" "DarkGray"

if (-not $SkipRecord) {
    $checkBuild = if ($stepsOk.build_iso) { "x" } else { " " }
    $checkIso = if ($stepsOk.iso_present) { "x" } else { " " }
    $checkVerify = if ($stepsOk.verify_stack) { "x" } else { " " }
    $checkHermes = if ($stepsOk.hermes_up) { "x" } else { " " }
    $checkCaps = if ($stepsOk.caps) { "x" } else { " " }
    $checkLadder = if ($stepsOk.ladder) { "x" } else { " " }
    $checkMcp = if ($stepsOk.mcp) { "x" } else { " " }

    $markVerify = if ($stepsOk.verify_stack) { "OK" } else { "FAIL" }
    $markCaps = if ($stepsOk.caps) { "OK" } else { "FAIL" }
    $markLadder = if ($stepsOk.ladder) { "OK" } else { "FAIL" }
    $markMcp = if ($stepsOk.mcp) { "OK" } else { "FAIL" }

    $body = @"
# QEMU E2E evidence - $stamp

- **Host:** $env:COMPUTERNAME
- **WSL:** $hasWsl
- **HostOnly:** $useHostOnly
- **ISO present:** $isoExists
- **Redox root:** $RedoxRoot

## Checklist

- [$checkBuild] build_iso
- [$checkIso] iso_present
- [$checkVerify] verify_stack
- [$checkHermes] hermes_up
- [$checkCaps] caps
- [$checkLadder] ladder
- [$checkMcp] mcp

## Aceite

| Criterio | Status |
|----------|--------|
| Host verify-stack | $markVerify |
| CapGate /caps | $markCaps |
| Escada FullLadder | $markLadder |
| MCP initialize | $markMcp |
| Guest qemu-guest-check | PENDING (precisa WSL + make qemu) |

## Log

Arquivo irmao: qemu-e2e-$stamp.log

## Proximo (guest)

1. Instalar WSL2 + toolchain Redox
2. tools/demo-qemu.ps1 (sem -HostOnly) ou tools/build-wsl.ps1
3. make qemu -> sh /usr/share/aios/qemu-guest-check.sh
4. Salvar saida como qemu-e2e-guest-$stamp.md
"@
    Set-Content -Path $evidencePath -Value $body -Encoding UTF8
    Set-Content -Path $logPath -Value ($script:LogLines -join "`n") -Encoding UTF8
    Write-Log "`nEvidence gravada: $evidencePath" "Green"
    Write-Log "Log: $logPath" "DarkGray"
}

Write-Log "`n=== Demo QEMU preparada (host gravavel) ===" "Green"
Write-Log "Guest: sh /usr/share/aios/qemu-guest-check.sh"
Write-Log "Guia: docs/DEMO-QEMU.md"

if ($startedStack) {
    Write-Log "Stack ainda ativa. Parar: Get-Job | Stop-Job; Get-Job | Remove-Job" "DarkGray"
}

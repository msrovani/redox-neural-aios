# QEMU no Windows nativo — Redox Neural AIOS
# Uso:
#   .\tools\qemu-win.ps1                         # descobre QEMU + imagem
#   .\tools\qemu-win.ps1 -Livedisk               # descompacta .zst em Downloads e sobe
#   .\tools\qemu-win.ps1 -Image C:\path\disk.iso
#   .\tools\qemu-win.ps1 -SmokeSeconds 15        # sobe e mata apos N segundos
param(
    [string]$Image = "",
    [switch]$Livedisk,
    [switch]$Harddrive,
    [int]$SmokeSeconds = 0,
    [string]$RedoxRoot = "",
    [string]$QemuPath = "",
    [int]$MemoryMb = 2048,
    [int]$Cpus = 2,
    [switch]$NoGraphic,
    [switch]$SmallLivedisk
)

$ErrorActionPreference = "Stop"

function Find-QemuExe {
    param([string]$Hint)
    if ($Hint -and (Test-Path $Hint)) { return (Resolve-Path $Hint).Path }
    $cmd = Get-Command qemu-system-x86_64.exe -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    $fallbacks = @(
        'C:\Program Files\qemu\qemu-system-x86_64.exe',
        'C:\Program Files (x86)\qemu\qemu-system-x86_64.exe',
        (Join-Path $env:LOCALAPPDATA 'Programs\qemu\qemu-system-x86_64.exe')
    )
    foreach ($p in $fallbacks) {
        if (Test-Path $p) { return $p }
    }
    throw "qemu-system-x86_64.exe nao encontrado. Instale via winget: winget install SoftwareFreedomConservancy.QEMU"
}

function Expand-Zst {
    param([string]$ZstPath, [string]$OutPath)
    if (Test-Path $OutPath) {
        Write-Host "Ja descompactado: $OutPath" -ForegroundColor DarkGray
        return $OutPath
    }
    Write-Host "Descompactando $ZstPath ..." -ForegroundColor Yellow
    $py = Get-Command python -ErrorAction SilentlyContinue
    if (-not $py) { throw "python necessario para .zst (pip install zstandard)" }
    $tmpPy = Join-Path $env:TEMP "redox-unzst-$PID.py"
    @(
        'import zstandard as zstd, pathlib, sys'
        'src, dst = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])'
        'dctx = zstd.ZstdDecompressor()'
        'with src.open("rb") as fi, dst.open("wb") as fo:'
        '    dctx.copy_stream(fi, fo, read_size=1024*1024, write_size=1024*1024)'
        'print(dst.resolve())'
    ) | Set-Content -Path $tmpPy -Encoding ASCII
    try {
        $printed = & python $tmpPy $ZstPath $OutPath 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) { throw "falha ao descompactar: $printed" }
    } finally {
        Remove-Item $tmpPy -Force -ErrorAction SilentlyContinue
    }
    if (-not (Test-Path $OutPath)) { throw "saida nao criada: $OutPath" }
    return $OutPath
}

function Find-LivediskZst {
    param([switch]$Smallest)
    $downloads = Join-Path $env:USERPROFILE "Downloads"
    $items = @(Get-ChildItem $downloads -Filter "redox_*livedisk*.iso.zst" -ErrorAction SilentlyContinue)
    if ($items.Count -eq 0) { return $null }
    if ($Smallest) {
        return $items | Sort-Object Length | Select-Object -First 1
    }
    return $items | Sort-Object LastWriteTime -Descending | Select-Object -First 1
}

function Find-BuiltImage {
    param([string]$Root, [switch]$PreferIso)
    $candidates = @(
        (Join-Path $Root "build\x86_64\desktop\harddrive.img"),
        (Join-Path $Root "build\x86_64\desktop\redox-live.iso"),
        (Join-Path $Root "build\x86_64\aios-minimal\harddrive.img"),
        (Join-Path $Root "build\x86_64\aios-minimal\redox-live.iso")
    )
    foreach ($c in $candidates) {
        if (Test-Path $c) { return $c }
    }
    return $null
}

if (-not $RedoxRoot) {
    $RedoxRoot = Join-Path (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)) "redox"
    if (-not (Test-Path $RedoxRoot)) {
        $RedoxRoot = Join-Path (Split-Path -Parent $PSScriptRoot | Split-Path -Parent) "redox"
    }
    # aios sibling layout: C:\DEV\redox-aios -> C:\DEV\redox
    $sib = Join-Path (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)) "redox"
    if (Test-Path $sib) { $RedoxRoot = $sib }
}

$qemu = Find-QemuExe -Hint $QemuPath
Write-Host "QEMU: $qemu" -ForegroundColor Cyan
& $qemu --version | Select-Object -First 1

if (-not $Image) {
    if ($Harddrive -or (-not $Livedisk)) {
        $built = Find-BuiltImage -Root $RedoxRoot
        if ($built) { $Image = $built }
    }
    if (-not $Image -or $Livedisk) {
        $zst = Find-LivediskZst -Smallest:$SmallLivedisk
        if (-not $zst) {
            throw "Nenhuma imagem. Passe -Image, ou coloque redox_*livedisk*.iso.zst em Downloads, ou build aios-minimal (WSL)."
        }
        $outIso = Join-Path $zst.DirectoryName ($zst.BaseName) # strips .zst -> .iso
        # BaseName of file.iso.zst is file.iso
        $Image = Expand-Zst -ZstPath $zst.FullName -OutPath $outIso
    }
}

if (-not (Test-Path $Image)) { throw "Imagem inexistente: $Image" }
Write-Host "Disk: $Image" -ForegroundColor Cyan

$args = @(
    "-machine", "pc",
    "-smp", "$Cpus",
    "-m", "$MemoryMb",
    "-device", "qemu-xhci",
    "-device", "usb-tablet",
    "-netdev", "user,id=net0",
    "-device", "e1000,netdev=net0"
)

$ext = [IO.Path]::GetExtension($Image).ToLowerInvariant()
if ($ext -eq ".iso") {
    $args += @("-cdrom", $Image, "-boot", "d")
} else {
    $args += @("-drive", "file=$Image,format=raw,index=0,media=disk")
}

if ($NoGraphic -or $SmokeSeconds -gt 0) {
    $args += @("-nographic", "-serial", "stdio", "-display", "none")
} else {
    $args += @("-serial", "stdio")
}

Write-Host "Cmd: `"$qemu`" $($args -join ' ')" -ForegroundColor DarkGray

if ($SmokeSeconds -gt 0) {
    Write-Host "Smoke: QEMU por ${SmokeSeconds}s ..." -ForegroundColor Yellow
    $outLog = Join-Path $env:TEMP "qemu-win-smoke-$PID.out.log"
    $errLog = Join-Path $env:TEMP "qemu-win-smoke-$PID.err.log"
    Remove-Item $outLog, $errLog -Force -ErrorAction SilentlyContinue
    $p = Start-Process -FilePath $qemu -ArgumentList $args `
        -RedirectStandardOutput $outLog -RedirectStandardError $errLog `
        -PassThru -NoNewWindow
    Start-Sleep -Seconds ([Math]::Min(4, $SmokeSeconds))
    if ($p.HasExited) {
        $err = if (Test-Path $errLog) { Get-Content $errLog -Raw } else { "" }
        $out = if (Test-Path $outLog) { Get-Content $outLog -Raw } else { "" }
        Write-Host "QEMU saiu cedo exit=$($p.ExitCode)`nERR:$err`nOUT:$out" -ForegroundColor Red
        exit 1
    }
    Start-Sleep -Seconds ([Math]::Max(0, $SmokeSeconds - 4))
    if (-not $p.HasExited) {
        Stop-Process -Id $p.Id -Force
        Write-Host "Smoke OK: QEMU ficou vivo ${SmokeSeconds}s (pid $($p.Id))." -ForegroundColor Green
        if (Test-Path $outLog) {
            $preview = Get-Content $outLog -TotalCount 30 -ErrorAction SilentlyContinue
            if ($preview) { Write-Host ($preview -join "`n") -ForegroundColor DarkGray }
        }
        exit 0
    }
    Write-Host "QEMU saiu durante smoke exit=$($p.ExitCode)" -ForegroundColor Yellow
    exit $p.ExitCode
}

Write-Host "Iniciando QEMU (feche a janela para sair)..." -ForegroundColor Green
& $qemu @args

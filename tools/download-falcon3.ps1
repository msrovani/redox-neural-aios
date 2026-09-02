# Baixa Falcon3-3B-Instruct para cortexd.
# Default: Q4_K_M (~2GB, melhor qualidade) via llama.cpp
# Uso leve: .\tools\download-falcon3.ps1 -Lite  (1.58bit BitNet)

param(
    [switch]$Lite,
    [switch]$GgufFallback
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
New-Item -ItemType Directory -Force -Path (Join-Path $Root "models") | Out-Null

pip install -q huggingface_hub 2>$null

if ($Lite) {
    $ModelsDir = Join-Path $Root "models\Falcon3-3B-Instruct-1.58bit"
    New-Item -ItemType Directory -Force -Path $ModelsDir | Out-Null
    $modelFile = Join-Path $ModelsDir "ggml-model-i2_s.gguf"
    if (-not (Test-Path $modelFile)) {
        Write-Host "Baixando Falcon3-3B-Instruct-1.58bit (TII)..."
        huggingface-cli download tiiuae/Falcon3-3B-Instruct-1.58bit-GGUF `
            ggml-model-i2_s.gguf --local-dir $ModelsDir
    }
    Write-Host "Lite model: $modelFile"
    Write-Host "REDOX_CORTEX_BACKEND=bitnet"
    exit 0
}

if ($GgufFallback) {
    $target = Join-Path $Root "models\Falcon3-3B-Instruct-IQ3_M.gguf"
    if (-not (Test-Path $target)) {
        huggingface-cli download bartowski/Falcon3-3B-Instruct-GGUF `
            Falcon3-3B-Instruct-IQ3_M.gguf --local-dir (Join-Path $Root "models")
    }
    Write-Host "REDOX_CORTEX_MODEL=$target"
    exit 0
}

# Default qualidade: Q4_K_M
$ModelsDir = Join-Path $Root "models"
$modelFile = Join-Path $ModelsDir "Falcon3-3B-Instruct-Q4_K_M.gguf"
if (-not (Test-Path $modelFile)) {
    Write-Host "Baixando Falcon3-3B-Instruct-Q4_K_M (qualidade default)..."
    huggingface-cli download bartowski/Falcon3-3B-Instruct-GGUF `
        Falcon3-3B-Instruct-Q4_K_M.gguf --local-dir $ModelsDir
}

Write-Host ""
Write-Host "Modelo default (qualidade): $modelFile"
Write-Host ""
Write-Host "Env vars:"
Write-Host "  `$env:REDOX_CORTEX_MODEL='$modelFile'"
Write-Host "  `$env:REDOX_CORTEX_BACKEND='llama-cpp'"
Write-Host "  `$env:REDOX_LLAMA_CLI='llama-cli'  # instale llama.cpp"
Write-Host ""
Write-Host "Variante leve: .\tools\download-falcon3.ps1 -Lite"

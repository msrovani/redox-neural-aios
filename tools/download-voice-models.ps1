# Baixa modelos STT (whisper) e TTS (piper) para pipeline Jarvis.
# Uso: .\tools\download-voice-models.ps1

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$WhisperDir = Join-Path $Root "models\whisper"
$PiperDir = Join-Path $Root "models\piper"
New-Item -ItemType Directory -Force -Path $WhisperDir, $PiperDir | Out-Null

pip install -q huggingface_hub 2>$null

# Whisper base (~150MB) — bom equilíbrio para dev
$whisperModel = Join-Path $WhisperDir "ggml-base.bin"
if (-not (Test-Path $whisperModel)) {
    Write-Host "Baixando whisper ggml-base..."
    huggingface-cli download ggerganov/whisper.cpp ggml-base.bin --local-dir $WhisperDir
}

# Piper pt-BR Faber medium
$piperOnnx = Join-Path $PiperDir "pt_BR-faber-medium.onnx"
$piperJson = Join-Path $PiperDir "pt_BR-faber-medium.onnx.json"
if (-not (Test-Path $piperOnnx)) {
    Write-Host "Baixando piper pt_BR-faber-medium..."
    huggingface-cli download rhasspy/piper-voices pt_BR/faber/medium/pt_BR-faber-medium.onnx --local-dir $PiperDir
    huggingface-cli download rhasspy/piper-voices pt_BR/faber/medium/pt_BR-faber-medium.onnx.json --local-dir $PiperDir
    # Reorganizar se veio em subpasta
    $nested = Join-Path $PiperDir "pt_BR\faber\medium\pt_BR-faber-medium.onnx"
    if (Test-Path $nested) { Copy-Item $nested $piperOnnx -Force }
    $nestedJson = Join-Path $PiperDir "pt_BR\faber\medium\pt_BR-faber-medium.onnx.json"
    if (Test-Path $nestedJson) { Copy-Item $nestedJson $piperJson -Force }
}

Write-Host ""
Write-Host "Modelos:"
Write-Host "  Whisper: $whisperModel"
Write-Host "  Piper:   $piperOnnx"
Write-Host ""
Write-Host "Instale binários no PATH:"
Write-Host "  whisper-cli  (llama.cpp / whisper.cpp releases)"
Write-Host "  piper        (https://github.com/rhasspy/piper/releases)"
Write-Host ""
Write-Host "Env vars:"
Write-Host "  `$env:REDOX_STT_ENGINE='whisper'"
Write-Host "  `$env:REDOX_TTS_ENGINE='piper'"
Write-Host "  `$env:REDOX_WHISPER_MODEL='$whisperModel'"
Write-Host "  `$env:REDOX_PIPER_MODEL='$piperOnnx'"

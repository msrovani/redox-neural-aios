# ADR-007: Cortex Daemon (`cortexd`)

- **Status:** Accepted
- **Lifecycle:** `fazendo`
- **Ideia:** #007
- **Sprint:** Fase 3
- **Depende de:** ADR-002, ADR-006

---

## Contexto

`Command::Chat` no Hermes precisa de inferência LLM/MoE. No neural-os-core isso vive no crate `cortex` (BitNet, GGUF, MoE). No Redox AIOS, inferência roda em userspace como daemon `cortexd`, desacoplado do Hermes.

## Decisão

### Modelo padrão: Falcon3-3B-Instruct-1.58bit (TII)

| Item | Valor |
|------|-------|
| Modelo | `tiiuae/Falcon3-3B-Instruct-1.58bit-GGUF` |
| Arquivo | `ggml-model-i2_s.gguf` |
| Backend primário | Microsoft BitNet (`run_inference.py`) |
| Fallback | llama.cpp GGUF (`REDOX_CORTEX_BACKEND=llama-cpp`) |
| Download | `tools/download-falcon3.ps1` |

### Componentes

| Crate/Daemon | Função |
|--------------|--------|
| `cortex-core` | `Falcon3Engine`, `AdaptiveEngine`, `CortexClient` |
| `cortexd` | Daemon TCP `127.0.0.1:7743` |
| `cortex` | CLI cliente |

### Integração Hermes (todas ações cognitivas)

- `Command::Chat` e linguagem natural → Falcon3
- Comandos `/ask` → Falcon3
- Comandos desconhecidos → Falcon3
- Skills `/help` e utilitárias (`echo`, `time`, `status`, `remember`, `recall`) permanecem locais

### Env vars

| Variável | Default |
|----------|---------|
| `REDOX_CORTEX_MODEL` | `models/.../ggml-model-i2_s.gguf` |
| `REDOX_CORTEX_BACKEND` | `auto` (1.58bit → bitnet) |
| `REDOX_BITNET_DIR` | `BitNet/` |
| `REDOX_CORTEX_ENGINE` | `stub` força stub |

### Fases futuras

- [ ] MoE routing + speculative decode
- [ ] Model hub integrado ao boot
- [ ] GPU/AVX backends nativos em Rust

## Verificação

- [x] `cortex-core` Falcon3 + AdaptiveEngine
- [x] `cortexd` + `cortex` CLI
- [x] Hermes roteia inteligência → cortexd
- [x] `tools/download-falcon3.ps1`
- [ ] inferência validada com modelo baixado no host

## Apreciação ADR-001

Materializa o pipeline cognitivo para linguagem natural — pré-requisito do Jarvis (STT→LLM→TTS).

# ADR-003: Pipeline Jarvis (voz)

- **Status:** Accepted
- **Lifecycle:** `fazendo`
- **Ideia:** #003
- **Sprint:** Fase 4
- **Depende de:** ADR-001, ADR-006, ADR-007

---

## Contexto

A premissa AIOS (ADR-001) exige experiência Jarvis: wake word → STT → Hermes → cortex → TTS. No neural-os-core isso vive em `jarbas::audio` (pipeline, wakeword, VAD, STT, TTS, barge-in). No Redox AIOS, o pipeline roda como daemon `voiced` em userspace.

## Decisão

### Componentes

| Crate/Daemon | Função |
|--------------|--------|
| `voice-core` | `VoicePipeline`, stubs wake/STT/TTS, clientes Hermes/eventd |
| `voiced` | Daemon TCP `127.0.0.1:7744` |
| `voice` | CLI cliente |

### Fluxo (stub Fase 4)

```
CLI/host text → voiced → VOICE_WAKE? → VOICE_STT
    → hermesd:7742 → cortexd (via Hermes Chat)
    → HERMES_RESPONSE → VOICE_TTS_START/END → [TTS stub]
```

Eventos publicados em `eventd:7740`:

- `VOICE_WAKE`, `VOICE_STT`, `VOICE_TTS_START`, `VOICE_TTS_END`
- (Hermes já publica `USER_INTENT`, `HERMES_RESPONSE`)

### Protocolo voiced (JSON-lines)

```json
{"cmd":"ping"}
{"cmd":"status"}
{"cmd":"utterance","text":"jarbas, que horas são"}
{"cmd":"say","text":"Olá, senhor."}
```

### Env vars

| Variável | Default | Uso |
|----------|---------|-----|
| `REDOX_VOICE_SOCKET` | `127.0.0.1:7744` | Bind do daemon |
| `REDOX_WAKE_WORD` | `jarbas` | Wake word (soul.toml) |
| `REDOX_VOICE_REQUIRE_WAKE` | `false` | Exigir wake word no host dev |

### Fase 4+ (atual)

- [x] `whisper.cpp` STT subprocess (`REDOX_STT_ENGINE=whisper`)
- [x] `piper` TTS subprocess + playback WAV (`REDOX_TTS_ENGINE=piper`)
- [x] `listen` com arquivo `.wav` → STT → Hermes → TTS
- [x] `tools/download-voice-models.ps1`
- [ ] captura MIC nativa (ALSA/Redox audio scheme)
- [ ] wakeword openWakeWord

## Verificação

- [x] `voice-core` pipeline + stubs
- [x] `voiced` + `voice` CLI
- [x] Integração hermesd + eventd
- [ ] áudio real MIC/SPK
- [ ] barge-in

## Apreciação ADR-001

Materializa o pipeline Jarvis declarado na premissa máxima — pré-requisito do `jarbasd` (Fase 5).

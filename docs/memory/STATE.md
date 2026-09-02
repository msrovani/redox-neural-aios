# STATE — Redox Neural AIOS

## Versão

- **Projeto:** Redox Neural AIOS v0.1.0
- **Data:** 2026-09-02
- **Roadmap:** [ROADMAP.md](../ROADMAP.md)

## Posição no roadmap

| Fase | Status host | Aceite OS |
|------|-------------|-----------|
| 0 | 🟡 overlay ✅, CI ✅ | ⏳ `make aios-minimal` (WSL) |
| 1 | 🟡 scheme bridges ✅ | ⏳ handler nativo |
| 2 | 🟡 HITL stub ✅ | ⏳ boot score QEMU |
| 4 | 🟡 audio bridge + barge-in stub | ⏳ MIC nativo |
| 6 | 🟡 | ⏳ demo gravável |

**Próximo:** `.\tools\build-wsl.ps1` ou build Linux → demo E2E Fase 6.

## Entregas desta sessão

- [x] `scheme_audio` — bridge `/scheme/audio` (MIC/SPK/VAD)
- [x] `barge_in` — interrupção TTS via VAD file
- [x] `voiced` — `listen` com `"scheme": true`
- [x] `hermes-core/hitl` — bloqueio `rm -rf` etc.
- [x] `boot_observe` — placar `=== BOOT SCORE ===`
- [x] `tools/verify-stack.ps1`, `tools/build-wsl.ps1`
- [x] `.github/workflows/ci.yml`
- [x] **29 testes** workspace verdes

## Verificação rápida

```powershell
.\tools\verify-stack.ps1          # testes + memory TCP + scheme
.\tools\build-wsl.ps1             # build Redox (requer WSL)
```

## Stack (host)

| Daemon | Porta |
|--------|-------|
| eventd | 7740 |
| sgdbd | 7741 |
| hermesd | 7742 |
| cortexd | 7743 |
| voiced | 7744 |
| jarbasd | 7745 |

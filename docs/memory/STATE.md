# STATE — Redox Neural AIOS

## Versão

- **Projeto:** Redox Neural AIOS v0.1.0
- **Data:** 2026-09-02
- **Roadmap:** [ROADMAP.md](../ROADMAP.md)

## Posição no roadmap

| Fase | Status host | Aceite OS |
|------|-------------|-----------|
| 0 | 🟡 overlay ✅, CI ✅ | ⏳ `make aios-minimal` (WSL) |
| 1 | 🟡 scheme bridges + URI nativo | ⏳ handler open() Redox |
| 2 | 🟡 HITL + CapGate scheme_caps | ⏳ caps nativas OS |
| 4 | 🟡 audio bridge + barge-in | ⏳ MIC nativo |
| 5 | 🟡 i18n parcial | ⏳ liborbital |
| 6 | ✅ demo-e2e.ps1 host | ⏳ demo gravável OS |
| 7 | 🟡 Onda 7 + Fase 2 factory | ⏳ QEMU escada gravável |

**Próximo:** handler `memory:` nativo + QEMU E2E completo.

## Escada cognitiva (ADR-010 revisada)

Exemplo *"qual a temperatura?"*: efêmera (1ª) → skill (3×) → wasm (runs maduros) → app wasmi (HITL).

| Degrau | Estado |
|--------|--------|
| 0 Efêmera ReAct+tools+SGDB | ✅ pipeline genérico + providers plugáveis |
| 1 SKILL.md | ✅ 7h |
| 2 WASM wasmi | 🟡 op-IR + boot reload |
| 3 App cookbook | 🟡 HITL + pkgutils build opt-in |

## Entregas recentes

- [x] **Fase 2 factory** — scheme URI/native prep, `scheme_caps`, cookbook pkgutils, `demo-qemu.ps1`
- [x] **Factory AIOS-aligned** — boot reload skills, ciclo O-P-A-V-R, eventos FACTORY_*, i18n, caps WASM
- [x] **Onda 7i** — pipeline efêmero + tool registry + providers HTTP (`REDOX_TOOLS_PROVIDERS`)
- [x] `tools/demo-ladder.ps1` — demo escada (3× intent → SKILL.md)
- [x] **ADR-001/002 host** — permission_gate, boot_observe+probe, BOOT_AI, backends honestos, scheme `aios:`
- [x] **Fix sgdbd Windows** — `REDOX_SGDB_PATH` (dir) → `mem.db` para `FileStorage`
- [x] **demo-e2e.ps1** verde no host (6 daemons + 5 passos)
- [x] i18n em hermes (help, HITL, offline) + voice (wake, barge-in)
- [x] `DataCollector` — pares Q/A no SGDB scope `voice`
- [x] `tools/demo-e2e.ps1` + `tools/start-stack.ps1`
- [x] **45 testes** workspace verdes

## Comandos

```powershell
.\tools\verify-stack.ps1
.\tools\demo-e2e.ps1
.\tools\demo-ladder.ps1 -WithNet
.\tools\demo-qemu.ps1          # build ISO + smoke guest
.\tools\start-stack.ps1 -KeepStack
```

## Stack

| Daemon | Porta |
|--------|-------|
| eventd | 7740 |
| sgdbd | 7741 |
| hermesd | 7742 |
| cortexd | 7743 |
| voiced | 7744 |
| jarbasd | 7745 |

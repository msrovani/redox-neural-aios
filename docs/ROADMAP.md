# Roadmap — Redox Neural AIOS

Documento canônico de **sequência de desenvolvimento**. Toda entrega nova deve respeitar a ordem abaixo — decisões de produto ([ADR-009](architecture/ADR-009-decisoes-produto.md)) ajustam *conteúdo*, não *fase*.

**Atualizado:** 2026-09-02 · **Versão alvo:** v0.1.0

---

## Princípio

```text
Fases 0→7  (stack cognitivo + integração)
Ondas A→C  (desktop Jarbas — paralelo a partir da Fase 5)
Tiers 1→4  (polish visual — Onda C / pós-v0.1)
```

Não pular fase. Hardening e integração QEMU vêm **antes** de features novas fora do plano.

---

## Decisões de produto (ADR-009) — invariantes no roadmap

| Tema | Valor |
|------|-------|
| Nome | **Redox Neural AIOS** |
| Desktop v0.1 | **Jarbas comanda**; apps **COSMIC convivem** |
| LLM default | **Falcon3-3B-Instruct** Q4_K_M |
| i18n | **pt-BR + en-US** desde v0.1 |
| Wake word | **`jarbas`** (configurável) |
| Repos | GitHub `msrovani/redox-neural-aios` + GitLab fork Redox |

---

## Fases de implementação

### Fase 0 — Fork & Governança

| Item | Status |
|------|--------|
| ADR-001, GOVERNANCE, IDEA_BANK, STATE | ✅ |
| `config/aios*.toml`, recipes, bootstrap | ✅ |
| ADR-002..007, ADR-009 | ✅ |
| `make aios-minimal` boot QEMU | ⏳ |
| Remotes git (GitHub + GitLab) | 🟡 GitHub configurado |
| CI build ISO | ⏳ |
| Overlay aplicado em `C:\DEV\redox` | ✅ |

**Aceite:** QEMU boot com `PRETTY_NAME="Redox Neural AIOS"`.

---

### Fase 1 — Fundação cognitiva

| Item | Status |
|------|--------|
| `agent-core`, `event-bus`, `skill-registry` | ✅ |
| `sgdbd` + neural-sgdb + CLI `memory` | ✅ |
| `eventd` + BOOT_PHASE | ✅ |
| Scheme `memory:` nativo | ⏳ |
| Scheme `memory:` file bridge | ✅ `memory-core` + sgdbd watcher |
| EventBus → scheme `chan:` | ✅ `event-bus/chan` + eventd |
| Agent wrappers (net, input) | ⏳ |

**Aceite:** `memory remember/recall` no Redox via scheme ou socket.

---

### Fase 2 — Hermes orquestrador

| Item | Status |
|------|--------|
| `hermes-core`, `hermesd`, skills builtin | ✅ |
| Intent routing → cortexd | ✅ |
| Boot observer stub → SGDB | 🟡 |
| HITL gate destrutivo | 🟡 stub |
| MCP server | ⏳ |
| `permission_gate` via capabilities | ⏳ |

**Aceite:** intent texto → skill ou LLM; ação destrutiva bloqueada com HITL.

---

### Fase 3 — Cortex inferência

| Item | Status |
|------|--------|
| `cortex-core`, `cortexd`, CLI `cortex` | ✅ |
| Falcon3 Q4_K_M (llama.cpp) | ✅ |
| BitNet 1.58bit (`-Lite`) | ✅ |
| Hermes Chat → cortexd | ✅ |
| Inferência no QEMU | ⏳ |
| Trinity MoE skeleton | ⏳ |
| Lazy load no boot | 🟡 |

**Aceite:** prompt PT-BR coerente no ambiente alvo (< 30s QEMU ou host documentado).

---

### Fase 4 — Pipeline de voz

| Item | Status |
|------|--------|
| `voice-core`, `voiced`, CLI `voice` | ✅ |
| STT whisper.cpp + TTS piper (host) | ✅ |
| Wake word `jarbas` (config) | ✅ |
| `listen` pipeline → hermes → cortex | ✅ |
| `wakewordd` / `sttd` / `ttsd` daemons separados | ⏳ |
| Scheme `audio` MIC/SPK nativo | ⏳ |
| Scheme `audio` file bridge | ✅ `scheme_audio` + barge-in |
| Barge-in E2E | 🟡 stub VAD file |
| Boot greeting TTS no OS | ⏳ |

**Aceite:** wake → STT → Hermes → Falcon3 → TTS → speaker no Redox.

---

### Fase 5 — Jarbas frontend (Onda A)

| Item | Status |
|------|--------|
| `jarbas-core`, `jarbasd`, CLI `jarbas` | ✅ |
| HUD terminal + Soul Mirror ASCII | ✅ |
| Boot greeting Falcon3 | ✅ |
| `i18n-core` + locales pt-BR/en-US | ✅ |
| Integração i18n em todos os daemons | 🟡 hermes + voice + jarbas HUD |
| `jarbas-ui` liborbital (compositor) | ⏳ |
| Chat scrollback, 5 themes | ⏳ |
| `jarbasd --login` substitui orblogin | ⏳ (Onda B plena) |

**Aceite:** boot → saudação visual+texto → chat interativo → orb reage.

Ver [ADR-004](architecture/ADR-004-jarbas-ui-strategy.md).

---

### Fase 6 — Integração Jarvis E2E

| Item | Status |
|------|--------|
| Wire host: voz + chat + memória | 🟡 |
| `jarbas-overlay` (Onda B convivência) | ✅ |
| SleepCycle / AutoLearn agents | ⏳ |
| DataCollector intent/response → SGDB | 🟡 voice scope |
| Cards: Terminal, Files, Weather | ⏳ |
| Demo gravável completa | 🟡 `tools/demo-e2e.ps1` host |
| `config/aios.toml` desktop completo | ⏳ |

**Aceite:** demo Jarvis gravável (boot, voz, terminal card, barge-in, memória entre sessões).

---

### Fase 7 — Polish & release v0.1

| Item | Status |
|------|--------|
| Boot observability (placar parseável) | ⏳ |
| SelfHeal / OTA skeleton | ⏳ |
| Benchmark latência wake→response | ⏳ |
| Testes QEMU headless | ⏳ |
| Redox Neural AIOS Book | ⏳ |
| **ISO `aios-v0.1.0`** | ⏳ |

**Aceite:** ISO publicável + critérios Fase 6 verdes.

---

## Ondas desktop (Jarbas UI)

Paralelas à Fase 5+, conforme [ADR-004](architecture/ADR-004-jarbas-ui-strategy.md) e ADR-009.

| Onda | Escopo | Fase ref. | Status |
|------|--------|-----------|--------|
| **A** | `jarbasd` + HUD terminal + Soul Mirror | Fase 5 | ✅ |
| **B** | Jarbas comanda; COSMIC convive (`jarbas-overlay`) | Fase 5–6 | 🟡 overlay ✅; shell login ⏳ |
| **C** | Compositor nativo + applets COSMIC | pós-v0.1 | ⏳ |

```
Usuário → Jarbas (voz/HUD/chat) → Hermes → Falcon3
              ↘ COSMIC apps (convivência v0.1)
```

---

## Tiers desktop (Onda C — polish visual)

Referência conceitual: neural-os-core ADR-0090. Aplicam-se **após** Fase 7 / Onda C.

| Tier | Foco | Exemplos |
|------|------|----------|
| **1 — Quick wins** | Performance 3–5× | glyph cache, dirty regions, dock visível |
| **2 — Polish** | UX | animações janela, scrollback chat, hover, waveform |
| **3 — Desktop real** | Funcionalidade | back buffers por janela, WM tiling, cards nativos |
| **4 — Transformacional** | Diferenciação | GPU compositor, mesh P2P, generative cards |

Não antecipar Tier 3–4 antes de Fase 6 E2E estável.

---

## Boot AIOS (Observe → Remember)

Ciclo transversal — hardening contínuo a partir da Fase 2:

| Etapa | Componente | Status |
|-------|------------|--------|
| Observe | `boot-observer` | 🟡 stub |
| Plan | `hermesd` + `cortexd` | 🟡 |
| Act | daemons Redox | ⏳ |
| Verify | boot score parseável | ⏳ |
| Remember | `sgdbd` | ✅ |

---

## Timeline (estimativa original)

| Marco | Semanas ~ | Demo |
|-------|-----------|------|
| Boot AIOS QEMU | 3 | `PRETTY_NAME` correto |
| Hermes + SGDB | 7 | chat texto |
| LLM local | 10 | prompt shell |
| Wake word | 14 | "jarbas" detectado |
| Jarvis E2E | 20 | voz completa |
| **v0.1 ISO** | 26 | demo gravável |

Scaffold host (Fases 0–6) adiantou protótipos; **critérios de aceite por fase** permanecem válidos.

---

## Próximo passo (sequência)

1. **Fase 0** — `bootstrap.ps1` + `make aios-minimal` + remotes git  
2. **Fase 1** — scheme `memory:` + bridge `chan:`  
3. **Fase 4** — scheme `audio` + barge-in no OS  
4. **Fase 6** — demo E2E gravável  
5. **Fase 7** — ISO v0.1.0  

Não iniciar Onda C / Tiers 3–4 antes de Fase 6 aceita.

---

## Referências

- [ADR-001 — Premissa máxima](architecture/ADR-001-aios-premissa-maxima.md)
- [ADR-002 — Arquitetura userspace](architecture/ADR-002-arquitetura-userspace.md)
- [STATE](memory/STATE.md) — verdade operacional atual
- [IDEA_BANK](memory/IDEA_BANK.md) — ideias ↔ fases

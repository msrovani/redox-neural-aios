# ADR-002: Arquitetura Userspace Cognitiva

- **Status:** Accepted
- **Lifecycle:** `fazendo`
- **Ideia:** #002
- **Sprint:** Fase 0–1
- **Depende de:** ADR-001 (Premissa Máxima)
- **Relacionado:** neural-os-core K³CHJ (fonte conceitual, não copiada no kernel)

---

## Contexto

O Redox AIOS adota a premissa [ADR-001](ADR-001-aios-premissa-maxima.md): IA desde o boot, sem modificar o microkernel Redox. Toda cognição vive em **userspace**, isolada por scheme IPC e capabilities.

O neural-os-core implementa cognição em monólito bare-metal (`no_std`, Ring 0). O Redox AIOS **porta os conceitos**, não o binário kernel.

## Decisão

### Camadas

```
┌─────────────────────────────────────────────────────────┐
│  Jarbas UI (jarbasd)          — shell AI, chat, orb     │
├─────────────────────────────────────────────────────────┤
│  Voz (voiced, sttd, ttsd, wakewordd) — pipeline Jarvis  │
├─────────────────────────────────────────────────────────┤
│  Cognição (hermesd, cortexd)   — routing, LLM, MoE      │
├─────────────────────────────────────────────────────────┤
│  Memória (sgdbd)              — neural-sgdb L0–L7       │
├─────────────────────────────────────────────────────────┤
│  Runtime (eventd)               — EventBus pub/sub        │
├─────────────────────────────────────────────────────────┤
│  Agent Core (agent-core, skill-registry) — tipos/shared │
├─────────────────────────────────────────────────────────┤
│  Redox Nativo                 — kernel, schemes, COSMIC │
└─────────────────────────────────────────────────────────┘
```

### Daemons

| Daemon | Porta | Crate | Status host | Aceite OS |
|--------|-------|-------|-------------|-----------|
| `eventd` | 7740 | `crates/daemons/eventd` | ✅ | ⏳ `chan:` |
| `sgdbd` | 7741 | `crates/daemons/sgdbd` | ✅ | ⏳ `memory:` |
| `hermesd` | 7742 | `crates/daemons/hermesd` | ✅ | ⏳ HITL |
| `cortexd` | 7743 | `crates/daemons/cortexd` | ✅ Falcon3 | ⏳ QEMU |
| `voiced` | 7744 | `crates/daemons/voiced` | ✅ whisper/piper | ⏳ audio scheme |
| `jarbasd` | 7745 | `crates/daemons/jarbasd` | ✅ Onda A | ⏳ login shell |

### EventBus

Fase 0: in-process `event-bus` crate (std, `HashMap` + `Mutex`).

Fase 1: bridge para scheme `chan:` do Redox — tópicos (`BOOT_PHASE`, `USER_INTENT`, `HERMES_RESPONSE`) como mensagens IPC.

### Scheme capabilities (futuro)

| Scheme | Função |
|--------|--------|
| `memory:` | remember/recall/health/curate (sgdbd) |
| `aios:` | registro de agentes, trust tokens |
| `chan:` | EventBus backend |

### Fork strategy

O repositório `redox-aios` é um **overlay** sobre upstream Redox:

1. `tools/bootstrap.ps1` — vincula ou clona Redox
2. `tools/apply-to-redox.ps1` — copia config, recipes, crates para `$REDOX_ROOT`
3. `tools/sync-upstream.ps1` — pull upstream + reaplica overlay
4. Build: `make CONFIG_NAME=aios-minimal` (ou `make aios-minimal` após include `mk/aios.mk`)

### Init sequence (aios-minimal.toml)

```
08_eventd  → EventBus + BOOT_PHASE
09_sgdbd   → memória cognitiva (stub)
20_orbital → desktop (upstream)
```

Fases futuras inserem `hermesd`, `cortexd`, `voiced`, `jarbasd` entre `09_sgdbd` e `20_orbital`.

## Implicações

- **Kernel intocado:** nenhuma mudança em `recipes/core/kernel` sem ADR dedicada.
- **Crates std:** daemons usam `std` + relibc, não `no_std`.
- **Recipes path:** `[source] path = "../../../crates/..."` relativo à recipe.
- **Sync upstream:** overlay reaplicado após cada pull; conflitos resolvidos em `redox-aios/`.

## Verificação

- [x] Crates `agent-core`, `event-bus` criados
- [x] Daemons `eventd`, `sgdbd` (stub) criados
- [x] Recipes + grupo `aios` criados
- [x] Config `aios-minimal.toml` com init.d
- [x] Scripts bootstrap/apply/sync
- [ ] `cargo check --workspace` no host
- [ ] Overlay aplicado em `C:\DEV\redox`
- [ ] `make aios-minimal` boota no QEMU

## Apreciação ADR-001

Esta arquitetura materializa o mandamento 1 (AIOS desde o boot) via init sequence userspace, e o mandamento 3 (caminho cognitivo) via EventBus + sgdbd, sem tocar no kernel.

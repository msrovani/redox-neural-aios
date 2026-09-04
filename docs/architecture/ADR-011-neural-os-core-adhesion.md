# ADR-011: Adesão neural-os-core → Redox Neural AIOS

- **Status:** Accepted
- **Lifecycle:** `fazendo`
- **Ideia:** #013
- **Sprint:** Fase 6–7 (contínuo)
- **Depende de:** ADR-001, ADR-002, ADR-006
- **Origem:** neural-os-core (K³CHJ, ADR-0088, agentes Hermes)

---

## Contexto

A adesão **não copia** o monólito bare-metal. Porta conceitos AIOS para userspace Redox (`std` + daemons + schemes), conforme ADR-001/002.

## Matriz de paridade

| Conceito neural-os-core | Destino RNAIOS | Status |
|-------------------------|----------------|--------|
| Premissa Máxima (0088) | ADR-001 | ✅ |
| Agent/Skill model | `agent-core` | ✅ |
| EventBus | `event-bus` + `chan:` bridge | 🟡 |
| neural-sgdb | `sgdbd` + `memory:` | 🟡 URI bridge |
| Hermes orquestrador | `hermesd` / `hermes-core` | ✅ |
| Cortex LLM | `cortexd` Falcon3 | 🟡 host |
| Jarbas + voz | `jarbasd` / `voiced` | 🟡 |
| Runtime App Factory (0059) | ADR-010 + Onda 7 | 🟡 |
| SelfHeal | `agent-core/self_heal` + `/selfheal` | ✅ userspace |
| SleepCycle (REPLAY…REFLECT) | `SleepCycleAgent` + `/lifecycle` | ✅ userspace |
| AutoLearn | `AutoLearnAgent` | ✅ userspace |
| Optimizer | `OptimizerAgent` | ✅ userspace |
| Boot observe | `boot_observe` + SelfHeal no boot | 🟡 |
| Trust tokens / CapGate | `scheme_caps` + `aios:/caps` + `redox_caps` | 🟡 namespace userspace |
| Trinity MoE | cortexd `trinity` | 🟡 skeleton |
| MCP server | `mcpd` stdio/TCP shim | ✅ |
| PackageHub | cookbook_bridge | 🟡 |
| SelfHeal PCI/firmware | — | ❌ N/A (kernel Redox) |
| Cranelift JIT skills | AWAITING_ISOLATION | ❌ gated |

## Decisão — o que portar agora

1. **Lifecycle agents reais** (não stubs `Pending`) — tick produz evidência SGDB + EventBus.
2. **SelfHeal userspace** — scan daemons TCP + backends honestos; propõe cura (HITL); sem PCI.
3. **Comandos Hermes** — `/lifecycle`, `/selfheal` (aliases `/agents`, `/heal`).
4. **Boot** — `hermesd` roda SelfHeal após `boot_observe`.

## O que NÃO portar

- Bitmap PMM / checkpoint heap (`k_ai::self_heal` Ring0)
- PCI VID-gated firmware scan
- Trust N2 bare-metal tokens (substituído por `REDOX_AIOS_CAPS`)
- `no_std` / irq locks

## Verificação

- [x] `SelfHealAgent` + `scan_stack`
- [x] SleepCycle 5 fases
- [x] AutoLearn gap detection
- [x] Optimizer score
- [x] `/lifecycle` `/selfheal` no router
- [x] Tick periódico lifecycle no daemon (`REDOX_LIFECYCLE_POLL_SECS`)
- [x] OTA skeleton HITL (`/ota check|approve`)
- [x] Caps OS scheme `aios:/caps` + `/caps`
- [x] Trinity MoE skeleton (`REDOX_CORTEX_MOE`)
- [x] Caps kernel Redox (userspace namespace) — `redox_caps` + `/caps ns|probe` + prep `REDOX_NSMGR_FD`
- [x] MCP server (`mcpd` stdio + TCP opcional :7746)
- [ ] Caps nativas via nsmgr FD (kernel/runtime)

## Apreciação ADR-001

Materializa mandamento 2 (auto-curar, auto-adaptar, auto-aprender) e mandamento 3 (caminho cognitivo) sem tocar no kernel Redox. CapGate espelha o modelo de namespace de schemes do Redox em userspace até existir FD nsmgr.

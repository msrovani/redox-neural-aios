# SESSION — Adesão neural-os-core lifecycle (2026-09-03)

## Objetivo

Prosseguir adesão ADR-001/002: portar SelfHeal + SleepCycle + AutoLearn + Optimizer do neural-os-core para userspace Redox Neural AIOS.

## Entregas

| Artefato | Função |
|----------|--------|
| `ADR-011` | Matriz de paridade neural-os-core → RNAIOS |
| `agent-core/self_heal.rs` | Scan daemons + backends → HealReport |
| `agent-core/lifecycle.rs` | Agentes reais (não stubs Pending) |
| `hermes-core/lifecycle_runner.rs` | Wire SGDB + EventBus |
| `/lifecycle` `/selfheal` | Comandos Hermes |
| hermesd boot | SelfHeal após boot_observe |

## Comandos

```powershell
hermes "/selfheal"
hermes "/lifecycle"
```

## Pendente

- Caps OS nativas
- Trinity MoE / MCP
- QEMU E2E gravável

## Env

| Variável | Default | Uso |
|----------|---------|-----|
| `REDOX_LIFECYCLE_POLL_SECS` | 600 | intervalo SelfHeal; `0` desliga |
| `REDOX_LIFECYCLE_FULL_EVERY` | 6 | a cada N ticks roda ciclo completo |
| `REDOX_OTA_CHANNEL` | stable | canal OTA |
| `REDOX_OTA_AVAILABLE` | 0 | `1` simula update disponível |

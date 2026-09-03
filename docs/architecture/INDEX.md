# Índice de ADRs — Redox Neural AIOS

Inventário canônico dos documentos em `docs/architecture/`. O **Status** registra a decisão no corpo da ADR; o **lifecycle** registra sua situação operacional no tree atual.

## Lifecycle

| Valor | Uso |
|-------|-----|
| `por_fazer` | Proposta aceita ou registrada, ainda não iniciada |
| `fazendo` | Implementação ativa |
| `completa` | Critérios atendidos |
| `substituida` | Preservada historicamente, superseded por outra ADR |
| `obsoleta` | Não seguir; mantida apenas como registro |
| `pesquisa` | Análise/research note, não decisão de implementação |

Status canônico no corpo: `Proposed | Accepted | Rejected | Superseded`.

## Inventário

| ID / arquivo | Status | Lifecycle | Ideias | Nota |
|---|---|---|---|---|
| `ADR-001-aios-premissa-maxima.md` | Accepted | `fazendo` | #001 | **Documento fundador** |
| `ADR-002-arquitetura-userspace.md` | Accepted | `fazendo` | #002 | Daemons, EventBus, overlay fork |
| `ADR-005-neural-sgdb-daemon.md` | Accepted | `fazendo` | #005 | sgdbd + memory CLI + socket IPC |
| `ADR-006-hermes-orquestrador.md` | Accepted | `fazendo` | #006 | hermesd + hermes-core + skills |
| `ADR-007-cortex-daemon.md` | Accepted | `fazendo` | #007 | cortexd + stub LLM + Hermes Chat |
| `ADR-004-jarbas-ui-strategy.md` | Accepted | `fazendo` | #004 | Ondas A→C (Fase 5–6) |
| `ADR-003-voice-pipeline-jarvis.md` | Accepted | `fazendo` | #003 | Fase 4 |
| `ADR-009-decisoes-produto.md` | Accepted | `completa` | #009 | v0.1 (invariantes) |
| `ADR-010-runtime-app-factory.md` | Accepted | `fazendo` | #010 | Escada efêmera→skill→wasm→app |
| `ADR-011-neural-os-core-adhesion.md` | Accepted | `fazendo` | #013 | Matriz de adesão + lifecycle |
| [ROADMAP.md](../ROADMAP.md) | — | `fazendo` | #010 | Fases 0→7 |
| ADR-008 (planejada) | — | `por_fazer` | #008 | Fase 0/7 |

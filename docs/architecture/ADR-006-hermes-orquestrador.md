# ADR-006: Hermes Orquestrador (`hermesd`)

- **Status:** Accepted
- **Lifecycle:** `fazendo`
- **Ideia:** #006
- **Sprint:** Fase 2
- **Depende de:** ADR-002, ADR-005

---

## Contexto

O Hermes do neural-os-core orquestra intents, skills, ReAct e publica `USER_INTENT` / `HERMES_RESPONSE` no EventBus. No Redox AIOS isso vira daemon userspace `hermesd`.

## Decisão

### Componentes

| Crate/Daemon | Função |
|--------------|--------|
| `skill-registry` | Registro de skills com policies |
| `hermes-core` | parse_command, router, ReAct trace, sgdb_client |
| `hermesd` | Daemon TCP `127.0.0.1:7742` |
| `hermes` | CLI cliente |

### Skills Fase 2

`echo`, `time`, `status`, `remember`, `recall`, `help`, `skills`

### Integração

```
CLI/shell → hermesd:7742 → HermesRouter → skill-registry
                        ↘ sgdbd:7741 (memória)
                        ↘ eventd:7740 (publish USER_INTENT / HERMES_RESPONSE)
```

### Boot observer

`boot_observe_and_remember()` grava evidência de boot no SGDB scope `boot`.

### HITL (stub Fase 2)

Skills destrutivas futuras usarão `hitl_required: true` + gate terminal. Não implementado nesta fase.

## Verificação

- [x] `hermes-core` parse + router
- [x] `hermesd` + `hermes` CLI
- [x] eventd publish remoto (7740)
- [x] boot_observe → sgdbd
- [ ] HITL approval UI
- [x] cortexd stub para `Command::Chat`

## Apreciação ADR-001

Hermes materializa mandamento 1 (IA no boot via boot_observe) e mandamento 3 (intent→skill→memória).

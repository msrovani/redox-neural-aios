# ADR-006: Hermes Orquestrador (`hermesd`)

- **Status:** Accepted (revisão 2026-09-02 — escada efêmera→skill→wasm)
- **Lifecycle:** `fazendo`
- **Ideia:** #006
- **Sprint:** Fase 2 + Fase 7
- **Depende de:** ADR-002, ADR-005, ADR-010

---

## Contexto

O Hermes do neural-os-core orquestra intents, skills, ReAct, self-evolve e publica `USER_INTENT` / `HERMES_RESPONSE` no EventBus. No Redox AIOS isso vira daemon userspace `hermesd`.

Esta revisão alinha o Hermes ao **Runtime App Factory** ([ADR-010](ADR-010-runtime-app-factory.md)): o router não é só "slash command → skill estática" — observa recorrência e alimenta a escada cognitiva.

---

## Decisão

### Componentes

| Crate/Daemon | Função |
|--------------|--------|
| `skill-registry` | Skills estáticas + `DynamicSkill` WASM |
| `hermes-core` | parse_command, router, ReAct, self-evolve, wasm_gen |
| `hermesd` | Daemon TCP `127.0.0.1:7742` |
| `hermes` | CLI cliente |

### Fluxo por tipo de intent

| Tipo | 1ª–2ª vez | ≥3 hits | Skill madura |
|------|-----------|---------|--------------|
| `/time`, `/echo` | builtin | builtin | builtin |
| Chat NL ("qual a temperatura?") | **Efêmera:** Cortex ReAct + SGDB | **Skill:** workflow SKILL.md | **WASM:** wasmi sandbox |
| `/weather_query` | skill registrada | idem | wasm se promovida |

### Ciclo ReAct (degrau 0 — efêmero)

```text
intent → parse → HITL gate → skill_observer
      → (sem skill?) → cortex complete + tools futuro
      → sgdbd remember → HERMES_RESPONSE
```

Todo intent passa por `observe_and_maybe_generate` ([ADR-010](ADR-010-runtime-app-factory.md)).

### Skills builtin (Fase 2+)

`echo`, `time`, `status`, `remember`, `recall`, `help`, `skills`, `factory`, `opir`, `promote`

Skills dinâmicas aparecem após self-evolve (ex.: `weather_query`).

### Integração

```
CLI/voz → hermesd:7742 → HermesRouter
                      ├→ skill-registry (static + dynamic wasm)
                      ├→ cortexd:7743 (ReAct / op-IR gen)
                      ├→ sgdbd:7741 (memória + usage)
                      └→ eventd:7740 (USER_INTENT / HERMES_RESPONSE / SELF_EVOLVE)
```

### Boot observer

`boot_observe_and_remember()` grava evidência de boot no SGDB scope `boot`.

### HITL

`permission_gate` via `hitl.rs` — bloqueia destrutivas; exige aprovação em `/promote` (pacote OS).

---

## Exemplo — roteamento "qual a temperatura?"

| Ocorrência | Router decide |
|------------|---------------|
| 1ª | `Command::Chat` → cortex (efêmera) → remember |
| 2ª | idem; `skill_observer` hits=2 |
| 3ª | `self_evolve` gera skill *(alvo 7h: SKILL.md)* |
| 4ª+ | match `weather_query` → executa skill (não ReAct livre) |
| após 3 runs OK | `skill_opt` → `.wasm` |
| HITL | `/promote weather_query approve` → cookbook |

---

## Verificação

- [x] `hermes-core` parse + router
- [x] `hermesd` + `hermes` CLI
- [x] eventd publish remoto (7740)
- [x] boot_observe → sgdbd
- [x] self_evolve + skill_observer integrados
- [x] HITL gate (permission_gate)
- [x] cortexd para `Command::Chat`
- [ ] `skill_gen` → SKILL.md (7h)
- [ ] Tool rede clima (7i)
- [ ] HITL approval UI

## Apreciação ADR-001

Hermes materializa mandamento 1 (boot_observe), mandamento 2 (auto-gerar via escada ADR-010) e mandamento 3 (intent→memória→skill).

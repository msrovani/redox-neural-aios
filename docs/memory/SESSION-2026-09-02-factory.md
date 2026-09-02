# SESSION — Onda 7 Runtime App Factory (2026-09-02)

## Objetivo

Alinhar a fábrica de apps efêmeras/definitivas às premissas Redox Neural AIOS (ADR-001/002/009/010).

## Escada implementada

| Degrau | Artefato | Gatilho |
|--------|----------|---------|
| 0 | Pipeline efêmero (6 tools) | 1ª–2ª intent |
| 1 | `SKILL.md` + trigger | hits ≥ 3 |
| 2 | `.wasm` wasmi | runs ≥ 3 @ 70% |
| 3 | recipe cookbook | `/promote approve` + HITL |

## Módulos novos

- `hermes-core`: `ephemeral`, `workflow`, `skill_gen`, `self_evolve`, `factory_boot`, `factory_cycle`, `wasm_gen`, `cookbook_bridge`, `tools/providers`
- `skill-registry`: `dynamic`, `skill_md`, `load_persisted_skills`
- `wasm-skill-runtime`: wasmi sandbox + CapGate
- `tools/demo-ladder.ps1`

## Eventos EventBus

- `FACTORY_BOOT`, `FACTORY_STAGE`, `FACTORY_REMEMBER`

## Paths OS (target Redox)

- Skills: `/var/lib/aios/skills`
- Staging recipes: `/usr/share/aios/recipes/skills-staging`
- Config: `/etc/aios/factory.toml`

## Env críticas

| Variável | Default | Uso |
|----------|---------|-----|
| `REDOX_TOOLS_NET` | 0 | HTTP providers |
| `REDOX_TOOLS_PROVIDERS` | — | `open_meteo`, `all` |
| `REDOX_HERMES_HITL` | 1 | Gate promoção |
| `REDOX_FACTORY_CAPS` | 1 | WASM host imports |
| `REDOX_SKILLS_DIR` | `/var/lib/aios/skills` | Persistência |
| `REDOX_RPC_TIMEOUT_MS` | 2000 | TCP daemons |

## Pendências (honestas)

- Scheme `memory:` nativo no target
- CapGate → scheme capabilities Redox (Fase 2)
- Build cookbook automático (pkgutils)
- QEMU E2E escada completa

## Demo

```powershell
.\tools\demo-e2e.ps1 -KeepStack
.\tools\demo-ladder.ps1 -WithNet
cargo run -p hermesd --bin hermes -- "/factory"
```

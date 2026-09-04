# SESSION 2026-09-04 — Caps kernel Redox (userspace namespace)

## Entrega

CapGate espelha o modelo Redox de **namespace de schemes** sem patch de kernel:

- `agent-core/redox_caps` — `NamespaceProfile`, probe, bootstrap `hermes`/`wasm_skill`, hint nsmgr
- Hermes `/caps ns [role]` e `/caps probe [role]`
- `hermesd` boot: `bootstrap_redox_ns()` após `bootstrap_caps()`
- Backend honesty: componente `caps` (Degraded até `REDOX_NSMGR_FD`)
- WASM: bitmask + `scheme_allowed` em `net_fetch`/`fs_read`; role `wasm_skill` sem `file` na base

## Comandos

```text
hermes "/caps list"
hermes "/caps ns"
hermes "/caps probe"
hermes "/caps bootstrap"
```

## Env

- `REDOX_CAP_ROLE` (default `hermes`)
- `REDOX_NS_SCHEMES` (escrito no bootstrap)
- `REDOX_NSMGR_FD` (futuro → Production)
- `REDOX_AIOS_CAPS` / `REDOX_AIOS_CAPS_ROOT`

## Próximo

MCP server · QEMU E2E gravável · nsmgr FD real

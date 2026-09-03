# SESSION — Caps OS + Trinity MoE (2026-09-03)

## Objetivo

ADR-011 próximo: Caps OS nativas (scheme bridge) + Trinity MoE skeleton.

## Entregas

| Artefato | Função |
|----------|--------|
| `agent-core/os_caps.rs` | CapStore em `/scheme/aios/caps/grants.json` |
| `scheme_caps` | Lê scheme → env → default |
| `/caps` | list / grant / revoke / bootstrap |
| hermesd boot | `bootstrap_caps()` |
| `cortex-core/trinity.rs` | route_intent multi-expert |
| `REDOX_CORTEX_MOE=1` | anota expert no complete |

## Uso

```powershell
hermes "/caps list"
hermes "/caps grant hitl_approve"
hermes "/caps grant pkg_install"
$env:REDOX_CORTEX_MOE="1"
# cortex complete anota [code]/general|…]
```

## Pendente

- MCP server
- Caps syscall kernel Redox
- QEMU E2E gravável

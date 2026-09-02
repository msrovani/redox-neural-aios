# SESSION — Fase 2b URI bridge (2026-09-02)

## Objetivo

Implementar canal `memory:` via URIs (`open/in/*.uri`) como passo intermediário até `open()` nativo Redox.

## Entregas

| Componente | Descrição |
|------------|-----------|
| `memory-core/scheme_open.rs` | `uri_to_body`, `rpc_uri`, `rpc_body` |
| `sgdbd/scheme_watcher.rs` | Poll `open/in/*.uri` |
| `permission_gate.rs` | `missing_scheme_grant`, `REDOX_AIOS_CAPS` |
| `wasm-skill-runtime` | `fs_read` retorna tamanho do arquivo (CAP_FS) |
| `recipes/groups/aios-skills-staging` | Meta-pacote staging |
| `verify-stack.ps1` | Teste URI bridge |
| `demo-qemu.ps1 -FullLadder` | Baseline host antes do guest |

## Fluxo URI

```text
MemoryClient (REDOX_MEMORY_SCHEME_NATIVE=1)
  → body_to_uri → open/in/{id}.uri
  → sgdbd poll_uri_open → handle_request
  → open/out/{id}.json
```

## Pendente

- Syscall `open("memory:…")` no kernel/scheme Redox
- Caps nativas OS (não só env CSV)
- `cookbook build` no WSL/Redox real
- Demo gravável QEMU end-to-end

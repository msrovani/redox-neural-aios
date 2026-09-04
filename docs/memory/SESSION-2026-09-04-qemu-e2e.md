# SESSION 2026-09-04 — QEMU E2E host gravável

## Contexto

WSL não instalado nesta máquina → guest `make qemu` bloqueado. Entrega: caminho **host gravável** + guest-check pronto para ISO.

## Entregas

- `tools/demo-qemu.ps1` — `-HostOnly`, evidence em `docs/memory/evidence/`, MCP+caps+escada
- `tools/qemu-guest-check.sh` + embed em `aios-minimal.toml` (caps/probe)
- `docs/DEMO-QEMU.md`
- `verify-stack.ps1` — `cargo --% test -j1 -- --test-threads=1` (evita race env caps)

## Uso

```powershell
.\tools\demo-qemu.ps1 -HostOnly
# suite completa opcional:
.\tools\demo-qemu.ps1 -HostOnly -FullCargoTest
```

## Pendente

Instalar WSL2 → `build-wsl.ps1` → `make qemu` → guest-check → `qemu-e2e-guest-*.md`

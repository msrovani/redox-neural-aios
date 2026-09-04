# SESSION 2026-09-04 — MCP server (`mcpd`)

## Entrega

Shim MCP JSON-RPC 2.0 (stdio + TCP opcional `:7746`) sobre `sgdbd`/`hermesd`.

- Tools: `health`, `remember`, `recall`, `hermes_intent`, `caps`, `backends`
- Resources: `aios://doctrine`, `aios://session`
- Docs: `docs/MCP.md`, `tools/mcp.aios.example.json`
- Recipe `recipes/aios/mcpd`, fleet entry em hermesd

## Uso

```powershell
.\tools\start-stack.ps1
cargo run -q -p mcpd --bin mcpd
# Cursor: copiar tools/mcp.aios.example.json → mcp.json
```

## Próximo

QEMU E2E gravável · nsmgr FD

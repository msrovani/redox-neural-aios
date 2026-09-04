# MCP — Redox Neural AIOS (`mcpd`)

Shim **Model Context Protocol** (JSON-RPC 2.0, stdio) sobre `sgdbd` e `hermesd` (ADR-011).

## Rodar

```powershell
# Stack precisa estar up (pelo menos sgdbd + hermesd)
.\tools\start-stack.ps1

# Stdio (Cursor / Claude)
cargo run -q -p mcpd --bin mcpd

# Stdio + TCP :7746
cargo run -q -p mcpd --bin mcpd -- --tcp
# ou REDOX_MCP_TCP=1
```

## Cursor

Exemplo em [`tools/mcp.aios.example.json`](../tools/mcp.aios.example.json):

```json
{
  "mcpServers": {
    "redox-aios": {
      "command": "cargo",
      "args": ["run", "-q", "-p", "mcpd", "--bin", "mcpd"],
      "cwd": "C:/DEV/redox-aios"
    }
  }
}
```

## Tools

| Tool | Backend |
|------|---------|
| `health` | sgdbd health + hermesd ping (`view=backends`) |
| `remember` / `recall` | sgdbd |
| `hermes_intent` | hermesd `intent` |
| `caps` | hermes `/caps …` |
| `backends` | hermesd `backends` |

## Resources

- `aios://doctrine` — instruções
- `aios://session` — sockets/ports cold-start

## Env

- `REDOX_SGDB_SOCKET` (default `127.0.0.1:7741`)
- `REDOX_HERMES_SOCKET` (default `127.0.0.1:7742`)
- `REDOX_MCP_SOCKET` (default `127.0.0.1:7746`)
- `REDOX_MCP_TCP=1`

# Demo QEMU E2E gravável — Redox Neural AIOS

Fluxo para **gravar evidência** (host + guest) da escada cognitiva e CapGate.

## Host (sempre — sem WSL)

```powershell
.\tools\demo-qemu.ps1 -HostOnly
# ou (auto HostOnly se WSL ausente):
.\tools\demo-qemu.ps1
```

Gera `docs/memory/evidence/qemu-e2e-<stamp>.md` com:

1. memory TCP smoke
2. stack `demo-e2e -KeepStack` (se hermes offline)
3. `/caps list` + `/caps probe`
4. escada inline: `/time` `/factory` `/evolve` `/promote list`
5. smoke `mcpd` initialize

Suite completa opcional: `-FullCargoTest` (chama `verify-stack.ps1`).

## QEMU no Windows (sem WSL)

QEMU 11+ em `C:\Program Files\qemu\` (pode nao estar no PATH):

```powershell
# Smoke: descompacta livedisk em Downloads e sobe 20s
.\tools\qemu-win.ps1 -Livedisk -SmallLivedisk -SmokeSeconds 20

# Interativo (janela + serial)
.\tools\qemu-win.ps1 -Livedisk

# Imagem AIOS buildada
.\tools\qemu-win.ps1 -Image C:\DEV\redox\build\x86_64\desktop\harddrive.img
```

Requer `pip install zstandard` para `.iso.zst`. Livedisks oficiais em Downloads nao incluem AIOS — so validam o hypervisor.

## Guest (quando ISO + QEMU)

1. WSL2 + [Podman build Redox](https://doc.redox-os.org/book/podman-build.html)
2. Overlay: `.\tools\bootstrap.ps1` / `apply-to-redox.ps1`
3. `.\tools\demo-qemu.ps1` (com WSL) **ou** `.\tools\build-wsl.ps1 -Target aios-minimal`
4. No WSL, a partir do root Redox:

```bash
make qemu
# login no guest
sh /usr/share/aios/qemu-guest-check.sh
```

5. Cole a saída em `docs/memory/evidence/qemu-e2e-guest-<stamp>.md`

## Aceite

| Item | Host | Guest |
|------|------|-------|
| `verify-stack` / memory URI | ✅ script | ✅ guest-check |
| Escada `/evolve` `/promote list` | ✅ FullLadder | ✅ guest-check |
| CapGate `/caps` | ✅ | ✅ |
| `PRETTY_NAME=Redox Neural AIOS` | — | ✅ grep os-release |
| MCP shim | ✅ initialize | opcional (stdio no guest) |

## Scripts

| Arquivo | Papel |
|---------|--------|
| `tools/demo-qemu.ps1` | Orquestra + grava evidence |
| `tools/qemu-guest-check.sh` | Smoke no guest |
| `tools/demo-ladder.ps1` | Escada ADR-010 |
| `tools/verify-stack.ps1` | Memory TCP/scheme/URI |
| `config/aios-minimal.toml` | Embute guest-check no ISO |

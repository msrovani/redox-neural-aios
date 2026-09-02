# Redox Neural AIOS

Fork do [Redox OS](https://www.redox-os.org) — **Sistema Operacional com Inteligência Artificial desde o boot**.

**Redox Neural AIOS** preserva o microkernel, scheme IPC, RedoxFS e o ecossistema COSMIC. A camada cognitiva — agentes, memória, orquestração, voz e **Jarbas** — roda em **userspace**, portada do [neural-os-core](https://github.com/msrovani/neural-os-core).

## Decisões de produto (v0.1)

| Tema | Decisão |
|------|---------|
| Nome | **Redox Neural AIOS** |
| Desktop | **Jarbas comanda**; apps **COSMIC convivem** no launcher |
| LLM | **Falcon3-3B-Instruct** (Q4_K_M, qualidade) |
| i18n | **pt-BR + en-US** desde v0.1 (`locales/`, `i18n-core`) |
| Wake word | **`jarbas`** (configurável em `soul.toml`) |

Ver [ADR-009](docs/architecture/ADR-009-decisoes-produto.md).

## Repositórios

| Remoto | URL |
|--------|-----|
| **GitHub** | https://github.com/msrovani/redox-neural-aios |
| **GitLab** | Fork Redox upstream: **Redox Neural AIOS** |

## Documentação

| Documento | Descrição |
|-----------|-----------|
| [ROADMAP](docs/ROADMAP.md) | **Sequência canônica** — fases 0→7, ondas A→C, tiers 1→4 |
| [ADR-001 — Premissa Máxima](docs/architecture/ADR-001-aios-premissa-maxima.md) | Mandamento fundador |
| [ADR-009 — Decisões de Produto](docs/architecture/ADR-009-decisoes-produto.md) | Nome, desktop, LLM, i18n |
| [STATE](docs/memory/STATE.md) | Posição atual no roadmap |
| [INDEX de ADRs](docs/architecture/INDEX.md) | Inventário |
| [GOVERNANCE](docs/GOVERNANCE.md) | Ciclo IDEA → ADR → código |

## LLM: Falcon3-3B-Instruct

```powershell
.\tools\download-falcon3.ps1          # Q4_K_M (qualidade, default)
.\tools\download-falcon3.ps1 -Lite    # 1.58bit (BitNet, leve)

$env:REDOX_CORTEX_MODEL="models\Falcon3-3B-Instruct-Q4_K_M.gguf"
$env:REDOX_CORTEX_BACKEND="llama-cpp"
cargo run -p cortexd --bin cortexd
```

## Quick start

```powershell
.\tools\bootstrap.ps1
.\tools\verify-stack.ps1      # testes + memory TCP/scheme
.\tools\demo-e2e.ps1          # demo Jarvis host (Fase 6)
.\tools\start-stack.ps1       # 6 daemons em background
cd C:\DEV\redox
make CONFIG_NAME=aios-minimal   # requer WSL/Linux
```

## Stack cognitivo

| Daemon | Porta |
|--------|-------|
| eventd | 7740 |
| sgdbd | 7741 |
| hermesd | 7742 |
| cortexd | 7743 |
| voiced | 7744 |
| jarbasd | 7745 |

## Licença

MIT (herdada do Redox upstream).

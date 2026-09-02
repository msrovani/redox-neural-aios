# ADR-005: neural-sgdb como daemon de memória (`sgdbd` + scheme `memory:`)

- **Status:** Accepted
- **Lifecycle:** `fazendo`
- **Ideia:** #005
- **Sprint:** Fase 1
- **Depende de:** ADR-001, ADR-002
- **Relacionado:** neural-os-core ADR-0091, neural-sgdb ADR-0008

---

## Contexto

O Redox AIOS precisa de memória cognitiva persistente para agentes (boot observer, Hermes, Jarvis). O [neural-sgdb](https://github.com/msrovani/neural-sgdb) v1.1.13 já oferece:

- Recall lexical (BM25) como default (ADR-0008)
- Scoping multi-agente
- FileStorage persistente
- Hits tipados (12 campos)

## Decisão

### Fase 1 — socket IPC + CLI `memory` (implementado)

| Componente | Função |
|------------|--------|
| `sgdbd` | Daemon com `FileStorage` em `/var/lib/sgdb` |
| `memory` | CLI cliente (`remember`, `recall`, `health`, `ping`) |
| Protocolo | JSON-lines sobre TCP `127.0.0.1:7741` |

**Comandos:**

```json
{"cmd":"remember","text":"...","scope":"boot"}
{"cmd":"recall","query":"...","scope":"boot","k":5}
{"cmd":"health"}
{"cmd":"ping"}
```

**Variáveis de ambiente:**

| Var | Default | Uso |
|-----|---------|-----|
| `REDOX_SGDB_PATH` | `/var/lib/sgdb` | Diretório do banco |
| `REDOX_SGDB_SOCKET` | `127.0.0.1:7741` | Bind/connect |

### Fase 1b — scheme `memory:` file bridge (implementado)

| Componente | Função |
|------------|--------|
| `memory-core` | Cliente unificado TCP + scheme file |
| `sgdbd` | Poll `REDOX_MEMORY_SCHEME_ROOT/in/*.json` |
| `memory` CLI | `REDOX_MEMORY_BACKEND=tcp\|scheme` |

**Variáveis:**

| Var | Default | Uso |
|-----|---------|-----|
| `REDOX_MEMORY_BACKEND` | `tcp` | `scheme` ativa bridge file |
| `REDOX_MEMORY_SCHEME_ROOT` | `/scheme/memory` | Diretório in/out |
| `REDOX_SGDB_SCHEME` | `1` | Watcher no sgdbd (`0` desliga) |

### Fase 2 — scheme `memory:` nativo Redox (em progresso)

| Componente | Função | Estado |
|------------|--------|--------|
| `memory-core/scheme_uri.rs` | URIs `memory:remember/recall/health` | ✅ contrato |
| `memory-core/scheme_native.rs` | Bridge até handler nativo | ✅ |
| `REDOX_MEMORY_SCHEME_NATIVE=1` | Default no target via factory.toml | ✅ config |
| Handler scheme Redox userspace | `open(memory:…)` → sgdbd | 🟡 URI bridge `open/` |

Bridge do scheme para o mesmo `SgdbService` interno do `sgdbd`.

### Dependência neural-sgdb

Path local no build: `../../../../neural-sgdb` (irmão de `redox-aios` em `C:\DEV\`).

Requisito de layout:

```
DEV/
├── neural-sgdb/
├── redox-aios/
└── redox/          # após overlay
```

## Verificação

- [x] `sgdbd` integra `neural-sgdb` FileStorage
- [x] `memory` CLI funcional
- [x] Teste `remember_recall_roundtrip`
- [x] Doctrine seed no primeiro boot (L3 lexical)
- [x] `memory-core` + bridge scheme file (Fase 1b)
- [x] `memory-core/scheme_uri` + `scheme_native` (contrato Fase 2)
- [x] `memory-core/scheme_open` URI bridge (`open/in/*.uri`)
- [ ] scheme `memory:` handler kernel Redox (`open()` syscall)
- [ ] Persistência validada no Redox QEMU (`tools/demo-qemu.ps1`)

## Apreciação ADR-001

Memória cognitiva materializa mandamento 3 (caminho cognitivo): todo fato vira conhecimento indexável com scope, recall lexical e checkpoint persistente.

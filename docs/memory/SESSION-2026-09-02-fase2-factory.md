# SESSION — Fase 2 Factory (2026-09-02)

## Objetivo

Fechar os quatro gaps do roadmap pós-Onda 7: scheme `memory:` nativo (prep), CapGate → capabilities, cookbook pkgutils, QEMU E2E smoke.

## Entregas

### 1. Scheme `memory:` nativo (prep userspace)

| Artefato | Função |
|----------|--------|
| `memory-core/scheme_uri.rs` | URIs `memory:remember/recall/health` |
| `memory-core/scheme_native.rs` | `REDOX_MEMORY_SCHEME_NATIVE`, `REDOX_OS_TARGET` |
| `memory-core/client.rs` | Prefere scheme quando native enabled |
| `config/aios-minimal.toml` | `/scheme/memory/in|out`, `profile.d/aios-env.sh` |
| `tools/qemu-guest-check.sh` | Smoke no guest (scheme, hermes, os-release) |

**Pendente:** handler `open(memory:…)` real no scheme Redox (upstream).

### 2. CapGate → scheme capabilities (host Fase 2a)

| Artefato | Função |
|----------|--------|
| `agent-core/scheme_caps.rs` | Parse `REDOX_AIOS_CAPS` (CSV) |
| `wasm_caps_from_grants()` | `CAP_NET`, `CAP_FS`, `CAP_LOG`, … |
| `skill-registry/dynamic.rs` | Grants na exec WASM |
| `wasm-skill-runtime/runtime.rs` | Stubs `aios::net_fetch` / `aios::fs_read` gated |

**Pendente:** mapear grants para capabilities nativas do scheme Redox.

### 3. Cookbook / pkgutils (opt-in)

| Artefato | Função |
|----------|--------|
| `cookbook_bridge.rs` | `PromoteResult`, `pkgutils_build_command`, `try_pkgutils_build` |
| `REDOX_COOKBOOK_BUILD=1` | Invoca `cookbook build` |
| `recipes/aios/skills/_template/recipe.toml` | Template promoção |
| `router.rs` | `/promote approve` retorna comando cookbook |

Promoção exige `pkg_install` ou `hitl_approve` em `REDOX_AIOS_CAPS`.

### 4. QEMU E2E smoke

| Artefato | Função |
|----------|--------|
| `tools/demo-qemu.ps1` | Build WSL + instruções QEMU + verify-stack |
| `tools/demo-ladder.ps1 -FullLadder` | Flag escada completa |
| Guest script no ISO | `/usr/share/aios/qemu-guest-check.sh` |

**Pendente:** demo gravável end-to-end (efêmera → skill → WASM → recipe no QEMU).

## Env novas

| Variável | Default | Uso |
|----------|---------|-----|
| `REDOX_MEMORY_SCHEME_NATIVE` | `0` host / `1` target | Modo scheme nativo |
| `REDOX_OS_TARGET` | — | `redox` no ISO |
| `REDOX_AIOS_CAPS` | — | CSV: `net,fs,pkg_install,hitl_approve,factory_exec` |
| `REDOX_COOKBOOK_BUILD` | `0` | `1` → `cookbook build` |

## Testes (host)

```
cargo test -p agent-core scheme_caps
cargo test -p memory-core roundtrip
cargo test -p hermes-core stages_recipe
cargo check --workspace
```

## Commit

Branch `main`, remote `github` → `msrovani/redox-neural-aios`.

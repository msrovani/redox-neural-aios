# ADR-009: Decisões de Produto (fundadoras)

- **Status:** Accepted
- **Lifecycle:** `completa`
- **Sprint:** v0.1
- **Depende de:** ADR-001

---

## Decisões registradas (2026-09-02)

| Tema | Decisão |
|------|---------|
| **Nome do fork** | **Redox Neural AIOS** |
| **Desktop** | **Jarbas comanda o COSMIC desde v0.1**, convivendo (não substitui apps COSMIC) |
| **LLM default** | **Falcon3-3B-Instruct** (qualidade — GGUF Q4_K_M via llama.cpp) |
| **LLM leve** | Falcon3-3B-Instruct-1.58bit (BitNet) — opcional |
| **Idioma** | **i18n desde v0.1** (pt-BR + en-US; extensível) |
| **Wake word** | **`jarbas`** por padrão, configurável em `soul.toml` / `REDOX_WAKE_WORD` |
| **Repositório GitHub** | `https://github.com/msrovani/redox-neural-aios` |
| **Repositório GitLab** | Fork Redox upstream: **Redox Neural AIOS** |

---

## Desktop — Jarbas + COSMIC

Jarbas é a **camada de comando cognitivo** do desktop desde v0.1:

- COSMIC apps (terminal, files, settings…) permanecem no launcher
- Jarbas overlay (`jarbas-overlay`) + `jarbasd` orquestram intents, voz e memória
- Futuro: applets COSMIC nativos (Onda C)

```
Usuário → Jarbas (voz/HUD/chat) → Hermes → Falcon3
              ↘ COSMIC apps (convivência)
```

## LLM — Falcon3-3B qualidade

| Perfil | Modelo | Backend |
|--------|--------|---------|
| **Default (qualidade)** | `Falcon3-3B-Instruct-Q4_K_M.gguf` | llama.cpp |
| Leve (1.58bit) | `ggml-model-i2_s.gguf` | BitNet |

Download: `tools/download-falcon3.ps1` (default Q4_K_M); `-Lite` para 1.58bit.

## i18n

- Crate `i18n-core` + `locales/{pt-BR,en-US}.json`
- Locale: `REDOX_LANG` ou `soul.toml` → `language`
- Todo texto visível ao usuário passa por `t("chave")`

## Wake word

- Default: `jarbas`
- Override: `/etc/jarbas/soul.toml` (`wake_word`) ou `REDOX_WAKE_WORD`
- Suporte a aliases futuros (ex: `jarvis`) via config

## Sequência de desenvolvimento

Esta ADR **não altera** a ordem do roadmap. Ajusta defaults e invariantes de produto dentro das fases já previstas.

Ver [ROADMAP.md](../ROADMAP.md) — fases 0→7, ondas A→C, tiers 1→4 (Onda C).

## Verificação

- [x] ADR-009 registrada
- [x] Defaults código + soul.toml + os-release
- [x] i18n-core scaffold
- [x] ROADMAP canônico publicado
- [ ] remotes git configurados pelo mantenedor
- [ ] push GitLab fork upstream

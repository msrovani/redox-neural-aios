# ADR-004: Jarbas UI Strategy (Onda A → C)

- **Status:** Accepted
- **Lifecycle:** `fazendo`
- **Ideia:** #004
- **Sprint:** Fase 5
- **Depende de:** ADR-001, ADR-006, ADR-007, ADR-003

---

## Contexto

Jarbas é o **comandante cognitivo** do desktop desde v0.1. COSMIC apps permanecem no launcher — convivência, não substituição (ADR-009).

## Decisão

### Ondas

| Onda | Entrega | Status |
|------|---------|--------|
| A | `jarbasd` TCP + HUD terminal ASCII + Soul Mirror simplificado | ✅ Fase 5 |
| B | `jarbas-overlay` + COSMIC convivendo — Jarbas comanda desde v0.1 | ✅ Fase 6 |
| C | Compositor Jarbas nativo (framebuffer) | futuro |

### Componentes Fase 5

| Crate/Daemon | Função |
|--------------|--------|
| `jarbas-core` | soul.toml, sessão, HUD, Soul Mirror, bridge Hermes |
| `jarbasd` | Daemon TCP `127.0.0.1:7745` |
| `jarbas` | CLI REPL interativo |

### Fluxo

```
jarbas CLI → jarbasd:7745 → hermesd → cortexd (Falcon3)
                         ↘ voiced (opcional, REDOX_JARBAS_VOICE=1)
                         ↘ eventd (JARBAS_CHAT_*)
```

### Protocolo jarbasd

```json
{"cmd":"chat","text":"..."}
{"cmd":"hud"}
{"cmd":"greet"}
{"cmd":"history"}
{"cmd":"voice","text":"jarvis, ..."}
{"cmd":"soul"}
{"cmd":"status"}
```

### Soul Mirror (Onda A)

Estado emocional derivado da resposta (`IDLE`, `THINK`, `SPEAK`, `ALERT`, `DREAM`) renderizado como orb ASCII no HUD.

### Boot

`jarbasd` emite boot greeting via Falcon3 (`REDOX_JARBAS_BOOT_GREET=1` default).

## Verificação

- [x] `jarbas-core` soul + session + hud + orb
- [x] `jarbasd` + `jarbas` REPL
- [x] Integração Hermes/Falcon3
- [x] overlay COSMIC/Orbital (`jarbas-overlay` + launcher + autostart)
- [ ] framebuffer Soul Mirror (Onda C)

## Apreciação ADR-001

Materializa o desktop Jarbas AI-first declarado na premissa máxima — ponto de entrada do usuário ao stack cognitivo.

# Governança documental — Redox Neural AIOS

Ciclo canônico que mantém intenção, decisão, execução e evidência sincronizadas.

## Ciclo obrigatório

```text
IDEA_BANK → ADR temática → plano de sprint → TODO → implementação + STATE
          → SESSION → check final (IDEA + ADR + STATE)
```

1. **IDEA_BANK** (`docs/memory/IDEA_BANK.md`): registrar toda ideia com status, destino e evidência.
2. **ADR temática** (`docs/architecture/`): agrupar ideias que impliquem decisão arquitetural.
3. **Plano de sprint**: escopo, ordem, critérios de aceite e riscos.
4. **TODO**: checklist executável com referências a IDEA/ADR.
5. **Implementação + STATE** (`docs/memory/STATE.md`): verdade operacional atual.
6. **SESSION** (`docs/memory/SESSION_NNN.md`): decisões, evidências, comandos, handoff.
7. **Check final**: sincronizar IDEA, INDEX, STATE, TODO, SESSION e [ROADMAP](ROADMAP.md).

## Roadmap

A sequência de fases (0→7), ondas desktop (A→C) e tiers de polish (1→4) é **canônica** em [docs/ROADMAP.md](ROADMAP.md). ADR-009 e decisões de produto não reordenam fases.

## Premissa máxima

Toda proposta é avaliada **primeiro** contra [ADR-001](architecture/ADR-001-aios-premissa-maxima.md). Se a solução "funciona mas ignora a IA", está incompleta.

## Campos cruzados mínimos

### IDEA_BANK

```text
ID | ideia | status | ADR (ou —) | sprint | evidência
```

Status: `✅` implementada · `🟡` em progresso · `⏳` pendente · `💰` custo alto · `❌` descartada

### ADR

```text
Status: Proposed | Accepted | Rejected | Superseded
Lifecycle: por_fazer | fazendo | completa | substituida | obsoleta
Ideia: #NNN
Sprint: NNN ou contínuo
```

### SESSION

```text
Data | sprint | ADRs tocadas | resumo | evidência (comandos, logs, PRs)
```

## Regras

- Fix pontual: pode seguir por `TODO + SESSION` sem ADR nova.
- Decisão arquitetural: exige ADR (própria ou temática existente).
- Toda ADR menciona como aprecia ADR-001.
- Workaround manual: exige IDEA com plano de auto-adaptação.

# ADR-001: Premissa Máxima — Redox AIOS-First (irrevogável, irretratável)

- **Status:** Accepted
- **Lifecycle:** `fazendo` (governa toda decisão a partir de 2026-09-02; operacionalização contínua)
- **Ideia:** #001
- **Sprint:** contínuo (toda sprint subsequente)
- **Substitui:** — (documento fundador do fork)
- **Relacionado:** neural-os-core ADR-0088 (fonte conceitual)

---

## Contexto

O **Redox AIOS** é um fork do Redox OS que transforma um sistema operacional microkernel maduro em um **AIOS** — Sistema Operacional com Inteligência Artificial **desde o boot**.

Isto não é uma feature, um pacote opcional nem um roadmap distante: é a **identidade do fork**. Toda decisão de engenharia — daemons, scheme IPC, filesystem, rede, desktop, pipeline de voz, política de memória — precisa ser avaliada primeiro sob esta lente, **antes** de qualquer consideração técnica isolada.

### Por que fork do Redox (e não reescrever do zero)

O [neural-os-core](https://github.com/msrovani/neural-os-core) provou o conceito AIOS em bare-metal monolítico (~26k LOC, K³CHJ). O Redox já oferece o que o neural ainda constrói:

| Capacidade | Redox (upstream) | neural-os-core |
|------------|------------------|----------------|
| Microkernel + isolamento real | ✅ scheme IPC | ❌ Ring 0 monolítico |
| Filesystem maduro | ✅ RedoxFS | 🟡 NeuralFS (em construção) |
| Desktop funcional | ✅ COSMIC/Orbital | 🟡 Jarbas compositor |
| Ecossistema de pacotes | ✅ pkgutils/recipes | ❌ |
| Compatibilidade POSIX/Linux | ✅ relibc | ❌ |
| IA nativa | ❌ | ✅ Cortex/Hermes/Jarbas |

**Decisão de fork:** manter o chassi Redox intacto e **portar a camada cognitiva** do neural-os-core para userspace.

### Escopo desta ADR

Esta ADR consolida, como premissa máxima irrevogável e irretratável, os cinco mandamentos que regem o comportamento do Redox AIOS, de todos os seus agentes (nativos, dev, IA) e de toda contribuição ao fork.

---

## Decisão — os cinco mandamentos

### 1. AIOS desde o boot

O Redox AIOS não é "Redox com um chatbot instalado". IA não é um serviço que roda por cima — é o **modo de operar**.

O boot, a descoberta de hardware, o init de daemons, o scheduler de agentes e o runtime são orientados por decisão inteligente desde o primeiro processo userspace:

```
init → eventd → sgdbd → boot-observer → hermesd → cortexd → voiced → jarbasd
```

Cada fase publica evidência no EventBus (`BOOT_PHASE`, `BOOT_AI`) e registra no neural-sgdb. O kernel Redox **não é modificado** para IA; a inteligência vive em userspace com scheme capabilities.

### 2. AI sempre, decisões HITL

O Redox AIOS usa IA em **toda decisão relevante**, sempre com **human-in-the-loop (HITL)**: interage, consulta, propõe e executa com supervisão.

O sistema se auto-tudo:

- **auto-adaptar** — detecta caminho subótimo e corrige em runtime
- **auto-curar** — SelfHeal detecta HW sem driver/skill e propõe solução
- **auto-upgrade** — OTA com gate HITL antes de aplicar
- **auto-gerar funcionalidades** — skills WASM on-demand via Hermes
- **auto-pesquisar soluções** — busca ativa quando bloqueado

Autônomo e automático, **sem jamais degradar segurança, HITL ou confiabilidade**.

Ações de alto impacto (instalação, update, execução de código não-confiável, mudança de política, operações destrutivas) **exigem aprovação humana** via `permission_gate` mapeado para scheme capabilities do Redox.

### 3. Toda decisão é tratada como caminho cognitivo

Nenhuma decisão ou caminho tomado fica sem tratamento:

```
inferência → adaptação → memorização → aprendizado → versionamento → auto-adaptação
```

Todo resultado vira conhecimento persistido:

| Destino | Conteúdo |
|---------|----------|
| **neural-sgdb** (`sgdbd`) | Fatos, episódios, decisões, pares intent/response |
| **SESSION** (`docs/memory/SESSION_NNN.md`) | Evidência, comandos, debug, handoff |
| **ADR** (`docs/architecture/`) | Decisões arquiteturais |
| **IDEA_BANK** (`docs/memory/IDEA_BANK.md`) | Ideias catalogadas com status |

O ciclo **Observe → Plan → Act → Verify → Remember** é obrigatório em boot, self-heal e resposta a incidentes.

### 4. Nada é simplesmente bypassado

Todo desvio, fallback ou workaround exige análise e pesquisa, gerando busca ativa por soluções, correções, melhorias e otimizações — sempre registrada no ciclo `IDEA → ADR → SESSION`.

Workarounds manuais (flags QEMU, patches temporários, "funciona no meu PC") são **sintomas de gap de auto-adaptação**, não destino final.

**Backends honestos:** caminho degradado, volátil ou emulado é reportado como tal (log + telemetria + HUD Jarbas), nunca fingido de saudável.

### 5. A busca incessante dos 10%

Todo procedimento persegue continuamente aqueles 10% de melhoria:

```
detectar → medir → decidir → otimizar → versionar
```

Sem nunca degradar segurança, HITL ou confiabilidade. Implementado por:

- `OptimizerAgent` — profiling e ajuste de parâmetros
- `SleepCycleAgent` — REPLAY → DREAM → CONSOLIDATE → PRUNE → REFLECT
- `AutoLearnAgent` — detecta gap → treina expert → registra no MoE

---

## Implicações operacionais

### Precedência de análise

Qualquer proposta (feature, fix, ADR, plano de sprint, PR) é avaliada **primeiro** contra esta premissa. Se a solução "funciona mas ignora a IA", ela está **incompleta por definição**.

Checklist obrigatório em toda proposta:

- [ ] A decisão passa por inferência ou adaptação?
- [ ] O resultado é memorizado (SGDB/SESSION/ADR)?
- [ ] HITL está presente para ações de impacto?
- [ ] Backends degradados são reportados honestamente?
- [ ] Existe caminho de auto-adaptação (não só workaround)?

### Arquitetura userspace (não kernel)

A premissa AIOS-first **não exige modificar o kernel Redox**. A camada cognitiva é composta por daemons userspace:

| Daemon | Função | Origem neural |
|--------|--------|---------------|
| `eventd` | EventBus sobre scheme `chan:` | `event-bus` crate |
| `sgdbd` | Memória cognitiva L0–L7 | `neural-sgdb` |
| `hermesd` | Orquestração, intent routing, HITL | `hermes` crate |
| `cortexd` | Inferência LLM/MoE | `cortex` crate |
| `voiced` | Pipeline voz (wake→STT→playback) | `jarbas::audio` |
| `jarbasd` | Shell AI, chat HUD, Soul Mirror | `jarbas::display` |

Isolamento entre agentes via **scheme capabilities** do Redox — natural fit para `permission_gate` e `CapGate`.

### Boot AIOS (Observe → Plan → Act → Verify → Remember)

| Fase | Componente | Papel |
|------|-----------|-------|
| Observe | `boot-observer` | Lê PCI/proc, gera DeviceTree |
| Plan | `hermesd` + `cortexd` | Plano de probe ordenado; Trust token |
| Act | daemons nativos Redox | Executa só o que o plano inclui |
| Verify | `boot-observer` | Placar parseável `=== BOOT SCORE ===` |
| Remember | `sgdbd` | Hidrata memória com evidência do boot |

### Pipeline Jarvis (voz)

A experiência Jarvis-like (STT + LLM + TTS) é requisito de produto, não opcional:

```
MIC → wakeword → VAD → STT → USER_INTENT → hermesd → cortexd → HERMES_RESPONSE → TTS → SPK
```

Com barge-in (interromper TTS falando), emoção (Soul Mirror), boot greeting e memória de pares intent/response no SGDB.

### Desktop Jarbas

O frontend Jarbas substitui gradualmente o fluxo `orblogin` → COSMIC como shell AI-first (Onda A: overlay → Onda B: shell → Onda C: compositor nativo). COSMIC apps permanecem acessíveis via cards.

### Sync com upstream Redox

O fork mantém sync periódico com upstream. Patches AIOS ficam isolados em:

- `config/aios*.toml`
- `recipes/aios/`
- `crates/` (workspace cognitivo)
- `docs/` (governança própria)

Mudanças no kernel Redox upstream são mergeadas; mudanças cognitivas não vão upstream sem ADR explícita.

---

## Relação com ADRs futuras

Esta ADR é a **raiz** de todas as decisões subsequentes do Redox AIOS:

| ADR planejada | Tema | Dependência |
|---------------|------|-------------|
| ADR-002 | Arquitetura userspace cognitiva | Esta ADR |
| ADR-003 | Pipeline Jarvis STT+LLM+TTS | Esta ADR |
| ADR-004 | Jarbas UI strategy (Onda A→C) | Esta ADR |
| ADR-005 | neural-sgdb como daemon de memória | Esta ADR |
| ADR-006 | Agent/Skill model + scheme IPC | Esta ADR |
| ADR-007 | Boot observability | Esta ADR |
| ADR-008 | Sync strategy com upstream Redox | Esta ADR |

Toda ADR futura **deve** mencionar como aprecia esta premissa.

---

## O que esta ADR NÃO decide

- Detalhes de implementação de crates (→ ADR-002)
- Escolha de modelos LLM/TTS/STT (→ ADR-003)
- Estratégia de UI Jarbas vs COSMIC (→ ADR-004)
- Modificações ao kernel Redox (proibidas sem ADR dedicada com justificativa extrema)
- Licenciamento de modelos ou dados de treino

---

## Verificação

- [x] ADR-001 redigida e aceita como documento fundador (2026-09-02).
- [ ] `config/aios.toml` boota com `PRETTY_NAME="Redox AIOS"`.
- [ ] Daemons cognitivos no init (`eventd`, `sgdbd`, `hermesd`).
- [ ] Boot observer registra evidência no SGDB.
- [ ] Pipeline Jarvis E2E funcional (wake → STT → LLM → TTS).
- [ ] Jarbas shell substitui `orblogin` no config AIOS.
- [ ] Toda nova ADR/TODO/SESSION menciona apreciação desta premissa.
- [ ] Todo workaround manual tem IDEA/ADR de auto-adaptação correspondente.

---

## Referências

- neural-os-core ADR-0088 (fonte conceitual)
- [Redox OS Book](https://doc.redox-os.org/book/)
- [neural-sgdb](https://github.com/msrovani/neural-sgdb)
- Plano de transformação Redox → Redox AIOS (sessão 2026-09-02)

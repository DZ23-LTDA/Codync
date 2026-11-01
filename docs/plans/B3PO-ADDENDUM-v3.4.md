# B-3PO — ADENDO v3.4: qualidade e usabilidade

Complementa o GOAL v3.2. **Não abre frente nova**: reordena o que existe, acrescenta dois itens ao gate do Release 1
(**9 → 11**) e cria a fila de usabilidade que vem depois.

Registrado em 31/10/2026 a partir do texto do dono.

## 0. Regra de entrega

**Um PR não é progresso até estar instalado e usado no computador do dono.**

Medido em 31/10: 38 PRs mergeados e o app instalado era a **v2.33.2** — nada do dia chegou à tela.

Regra: ao fechar um conjunto de PRs, **bumpar, publicar e atualizar o app instalado**. O relatório de sessão diz a versão
instalada e a data do último `dev` contido nela. "Entregue" sem versão instalada é `NOT_STARTED`.

## 1. Gate do Release 1 passa a ter 11 itens

Os 9 da seção 18 do v3.2 continuam. Entram:

**10. Ciclo de entrega funcionando**
- Release gate separado do upstream (seção 15 do v3.2) e pipeline do fork publicando.
- Bump + tag + `WhatsNew` + release.
- **Update in-app testado no Windows**: instalar a versão anterior, sair, reabrir, ver a atualização aplicada, conferir que
  `~/.codync` sobreviveu.
- Métrica: *tempo entre merge em `dev` e app instalado atualizado*. Hoje: **infinito**.
- Aceite: um ciclo completo observado, com URL de release e versão exibida no app.

**11. Cancelar e destravar**
- Turno travado, processo órfão, agente que não responde: detectar (watchdog por tempo sem evento), oferecer **cancelar** e
  explicar o que aconteceu.
- Aceite: bot preso em "trabalhando" pode ser cancelado pela UI, o processo é morto, o chat registra o cancelamento com
  motivo; turno que morre no meio não deixa o bot travado.

## 2. Medir antes de melhorar (obrigatório)

Três medidas reais em `docs/`, sem estimativa:

1. **Primeiro uso em máquina limpa** (VM Windows, sem Node, sem agente): tempo até a primeira resposta e o que a tela mostra
   quando **nenhum agente é detectado**.
2. **Cobertura pt-BR medida** com o scanner corrigido, por cliente — ver `docs/plans/ptbr-coverage-report.md`.
3. **Efeito de um travamento**: turno cujo agente não responde — o que o usuário vê e como sai.

Nenhuma meta deste adendo vale sem esses três números.

## 3. Release 1.5 — usabilidade (depois do gate, nesta ordem)

| # | Item | Estimativa | Aceite |
|---|---|---|---|
| U1 | **Memória visível e corrigível** | 1 semana | encontrar preferência errada e corrigir sem terminal; bot usa a versão corrigida no turno seguinte |
| U2 | **Aprovação como tela principal** | 3–5 dias | aprovar 3 ações seguidas sem rolar chat; recusa registra motivo; histórico mostra quem e quando |
| U3 | **Diagnóstico por turno** | 3 dias | abrir a falha e entender a causa sem o *Full conversation*; mesmo dado em desktop, iOS e TUI |
| U4 | **Preview** | — | entra **depois** de U1–U3; dono já declarado |

## 4. Fora de escopo agora (cortar, não adiar com data)

- **Marketplace de bots**; **canais novos** (Feishu, QQ, WeChat — só Slack/Discord);
- **Model Gateway, roteamento por custo, RBAC/multi-tenant, SLO, painéis operacionais, temas/wallpapers/painéis
  arrastáveis**: ficam na seção 20 do v3.2, sem data.

## 5. Processo

1. No máximo **3 PRs abertos**.
2. **Uma release por conjunto fechado**, não por PR.
3. `dev` protegida ou, enquanto não estiver, CI verde + revisão antes do merge.
4. **Sessão com prazo**: ao fim do orçamento, árvore limpa, checkpoint escrito, versão instalada informada.
5. Nada de "encontrei uma coisa melhor" no meio do gate — ideia nova vai para a seção 20 com uma linha.

## 6. Métricas de sucesso deste adendo

| Medida | Antes (31/10) | Meta |
|---|---|---|
| Merge até app instalado atualizado | infinito (app em v2.33.2) | ≤ 1 dia |
| Strings visíveis sem tradução | 271+ (subcontado) | 0 no escopo liberado |
| Primeiro uso em máquina limpa | não medido | < 3 min até a primeira resposta |
| Bot travado | sem saída conhecida | cancelar + diagnóstico em 1 clique |
| Aprovação | cartão no meio do chat | tela dedicada com payload e histórico |

O relatório de sessão passa a incluir **a versão instalada**, além dos três números da seção 23 do goal.

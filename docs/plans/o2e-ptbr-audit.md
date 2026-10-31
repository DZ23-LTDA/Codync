# O2-E (pt-BR): auditoria antes de traduzir

Medido no checkout atual, não estimado.

## O que existe hoje

- **Não há i18n.** Nenhum catálogo de mensagens, nenhuma pasta de locale no código dos clientes — só artefatos de build.
  Traduzir string a string espalhada nos arquivos seria criar dívida: cada texto novo volta em inglês e a mistura de
  idiomas é exatamente o que o critério de O2-E proíbe.
- **Desktop** (`apps/desktop/src`, 347 arquivos `.ts`/`.tsx`):
  - **46** nós de texto JSX (`>Texto<`) — poucos porque a regra de UI do projeto é botão só com ícone;
  - **253** atributos visíveis (`placeholder`, `title`, `aria-label`);
  - **1.638** literais candidatos a copy no renderer (inclui nomes de campo de API — a triagem é parte do trabalho).
- **TUI** (`host/src/tui`, 80 arquivos): **2.217** literais, a maioria teclas e formatos; a copy real é menor e precisa
  da mesma triagem.
- **iOS e Android** não foram medidos aqui: exigem Xcode e Android SDK, que este ambiente não tem
  (`BLOCKED_BY_EXTERNAL_DEPENDENCY`).

## Decisão de arquitetura (antes de traduzir)

1. **Catálogo de mensagens por cliente**, com `en-US` como chave de origem e `pt-BR` como tradução: `t("chat.empty")`.
   Uma chave sem tradução cai no inglês e é listada por um teste — nada de string solta.
2. **Formatação por locale** (data, hora, número, moeda) via `Intl` no desktop, `Locale` no Android, `Date.FormatStyle`
   no iOS e a mesma regra na TUI. Nada de concatenar data formatada à mão.
3. **Teste de cobertura**: falha se uma chave usada no código não existir no catálogo, ou se `pt-BR` tiver chave órfã.
4. **Glossário fixo** para não traduzir o que não deve: *bot*, *agente*, *run*, *harness*, *skill*, *conector*, *MCP*,
   *token*, *workspace* permanecem; "settings" → "configurações", "approve" → "aprovar", "budget" → "orçamento",
   "run" (execução) → "execução" quando for a coisa, "run" quando for o registro técnico.
5. **Não traduzir** identificadores de API, nomes de método, valores persistidos, nomes de modelo e mensagens de log.

## Ordem de entrega proposta

1. Fundação de i18n no desktop + catálogo `en-US` extraído de **diálogos, erros e estados vazios** (onde está a maior
   parte da copy) — primeiro PR.
2. Tradução `pt-BR` dessas mesmas telas + formatação de data/número (segundo PR).
3. TUI (catálogo e tradução) — terceiro PR.
4. iOS e Android quando houver ambiente para compilar; até lá ficam `BLOCKED_BY_EXTERNAL_DEPENDENCY`, não "feitos".

## Riscos

- Traduzir só o desktop quebra a paridade que o repositório exige; por isso a TUI entra antes dos apps nativos, que
  dependem de ambiente.
- Texto em pt-BR é ~20% mais longo: cada tela traduzida precisa de inspeção visual, não só de teste.

## Próxima tarefa concreta

PR 1: `apps/desktop/src/renderer/i18n/` com catálogo tipado, `t()` e o teste de cobertura; extrair a copy dos
diálogos/erros/estados vazios do desktop para `en-US`. Sem mudança visível ainda — é a fundação que impede a mistura.

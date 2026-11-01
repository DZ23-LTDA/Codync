# Telas abertas no app e o que cada uma revelou

Todas pelo demo (`src/renderer/demo`), que roda o renderer real com um host de memória. Capturas em
`evidencias/demo-*.png`. O placar: **cada tela aberta revelou exatamente um defeito que nenhum medidor via**.

| # | Tela | Como chegar | Defeito revelado | Estado |
|---|---|---|---|---|
| 1 | Conversa | inicial | chrome já em pt (sem defeito) | ok |
| 2 | Busca sem resultado | lupa + `zzzz` | "No matching bots" | corrigido |
| 3 | Extensões → Agentes | rodapé → Extensões | abas Agents/Connectors/Skills e "Installed" | corrigido |
| 4 | Extensões → Credenciais | Extensões → Credenciais | "Save sign-in", "Connect a shared vault" | corrigido |
| 5 | Menu de contexto do bot | clique direito no item | "Pin" | corrigido |
| 6 | Painel da conta | botão da conta | "Support" | corrigido |
| 7 | Configurações → Geral | conta → Configurações | "This Mac", "Personal", "Follow system" | corrigido |
| 8 | Configurações → Computadores | Configurações | — | cabeçalho verificado em pt |
| 9 | Configurações → Rede | Configurações | — | idem |
| 10 | Configurações → Acesso ao computador | Configurações | — | idem |
| 11 | Configurações → Atualizações | Configurações | "Check for updates", "Updates are available in release builds." | corrigido |
| 12 | Menu de contexto do bot (2ª leitura) | clique direito no item | — (Fixar, Mover para a seção, Editar perfil…, Tornar o agente principal) | ok |
| 13 | Editor de bot | menu → "Editar perfil…" | — (Perfil, Avatar, Nome, Agente, Computador em pt) | ok |

## O que as 8–11 provam, e o que não provam

Provam que **a navegação existe e o cabeçalho está em português** ("Computadores e aparelhos", "Rede", "Acesso ao
computador", "Atualizações"), com **0 erros de console**.

Não provam que o **corpo** de cada uma está traduzido: a extração de texto parou na barra lateral nas quatro. Corpo das
telas 8–11 continua **não inspecionado**, e é assim que fica registrado.

## O padrão que se repetiu

Os sete defeitos das telas 2–7 estavam todos em texto que **não passa por `tr()` no próprio arquivo**: atributo de componente,
array de configuração, ternário, ou variável passada ao `tr` (caso de `This Mac`/`Follow system`). Os quatro medidores do
projeto cobrem literais e expressões JSX; nenhum segue variável, e é exatamente aí que estavam os defeitos.

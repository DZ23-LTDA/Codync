# D2 — os 11 itens do gate, um a um

Decisão de escopo do proprietário pendente para os itens marcados `REQUIRED_FOR_RELEASE_1` / `DEFERRED_BY_OWNER`.
Enquanto ela não vem, o desenvolvimento independente continua. Estados comprovados no SHA `1f63bb3a`.

| # | Item | Estado comprovado | Evidência | Dependência | Impacto de adiar | Risco | Recomendação |
|---|---|---|---|---|---|---|---|
| 1 | scanner de i18n corrigido e recontado | **VERIFIED** | `l10n-missing` decodifica escapes; 815 frases em `tr()`, 0 sem `pt.json`; `pt` 1673 chaves/100% | — | nenhum: já protege as outras releases | baixo | `REQUIRED_FOR_RELEASE_1` |
| 2 | release do fork separado do upstream | **VERIFIED** | `release-gate.py` recusa repositório **e tap** de terceiro; 64 actions pinadas por SHA | — | publicaria em conta alheia | alto se cair | `REQUIRED_FOR_RELEASE_1` |
| 3 | banco ausente, vazio, válido, danificado, migrado, interrompido, recuperado | **VERIFIED** | `write_interruption_e2e.rs`, `database_damage_e2e.rs`; suíte em 3 SOs | — | perda de dados silenciosa no primeiro crash real | alto se cair | `REQUIRED_FOR_RELEASE_1` |
| 4 | instalação limpa, update e rollback em três sistemas | **TESTED** | Windows ponta a ponta (#212): instala 14,4 s, abre, desinstala 6,3 s. **Update e rollback não** (build sem `latest.yml`); macOS/Linux não mensuráveis aqui | máquina macOS + autorização de publicação | usuário instala e não consegue atualizar | médio | `REQUIRED_FOR_RELEASE_1` para instalar/desinstalar; **update fica `BLOCKED_BY_EXTERNAL_DEPENDENCY`** até haver feed |
| 5 | primeiro uso em menos de 3 min, QR, reconexão, desinstalar | **IN_PROGRESS** | 22,8 s do instalador à janela usável; QR **emitido**; reconexão provada na TUI (#208, #209); desinstalar 6,3 s | celular + firewall liberado + **um agente CLI instalado** | primeiro uso não termina numa resposta | médio | `REQUIRED_FOR_RELEASE_1`; as duas metades restantes são `BLOCKED_BY_EXTERNAL_DEPENDENCY` |
| 6 | erros humanos, estados vazio e offline | **IN_PROGRESS** | 6 testes de store (#214–#222) + estados da TUI (#206, #207) + capturas do desktop; `humanError` parou de mostrar `ECONNREFUSED` (#215) | builds iOS/Android para as telas do celular | tela de erro mostra detalhe técnico | baixo | `REQUIRED_FOR_RELEASE_1` |
| 7 | pt-BR no escopo liberado, inspecionado | **TESTED** | catálogo pt completo (1673); telas do desktop inspecionadas; **telas internas do celular sem inspeção** | pareamento (firewall) para chegar às telas internas | texto misturado em português | baixo | `REQUIRED_FOR_RELEASE_1` |
| 8 | performance medida | **VERIFIED** | `/health` p50 0,9/p95 1,3 ms; primeiro token p95 17 ms (1 bot) e 1211–1212 ms (8 bots); CPU 28,6% de um núcleo; desktop 922 ms; Android frio 2493 ms | — | nenhum | baixo | `REQUIRED_FOR_RELEASE_1` |
| 9 | checkpoint, bump, WhatsNew, tag | **IN_PROGRESS** | checkpoint agora cobre 03/11 (#220); versão 2.34.1 nos três manifestos; **`project.pbxproj` ainda 2.34.0**; tag `v2.34.1` não criada | `xcodegen` num Mac + autorização de tag | release sem identificação consistente | médio | `REQUIRED_FOR_RELEASE_1`; o que falta é mecânico e externo |
| 10 | ciclo de entrega: bump, tag, publicar, update no app | **BLOCKED_BY_EXTERNAL_DEPENDENCY** | `v2.34.0` publicada com upload **incompleto** (sem `latest.yml`, `latest-mac.yml`, quatro `*.update.json`) | autorização de publicação + feed | app não atualiza; é o item que impede "release entregue" | alto | `BLOCKED_BY_EXTERNAL_DEPENDENCY` até o dono autorizar publicação |
| 11 | cancelar turno travado | **VERIFIED** | Stop exercitado em janela real com agente que não responde: turno fecha, "Stopped." em 17 s | — | usuário fica preso num turno | alto se cair | `REQUIRED_FOR_RELEASE_1` |

## O que isso quer dizer para o fechamento

- **5 itens `VERIFIED`** (1, 2, 3, 8, 11): nada a fazer.
- **2 itens `TESTED`** (4, 7) e **3 `IN_PROGRESS`** (5, 6, 9): todos com trabalho conhecido e rastreável, nenhum com causa raiz aberta.
- **1 item bloqueado** (10) por decisão de publicação do dono.
- Nenhum P0 conhecido em aberto. Os P1 restantes dependem de celular, firewall, agente CLI, Mac ou autorização de publicação.

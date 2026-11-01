# Release 1 — o que falta, e do que cada coisa depende

Escrito em 31/10/2026, 22:20Z, a partir do que foi medido. Não é um plano novo: é a fila do goal v3.2 (seção 18), com o
que cada item espera.

## Medido agora

| | |
|---|---|
| `dev` | `85671d9f` |
| itens do gate | 2 com implementação e teste (1, 2), 1 parcial (3), 1 em progresso (6) |
| strings fora da tradução | **314** (`node tools/l10n-unwrapped.mjs`) |
| telas com coleção sem estado vazio | **36** de 63 (`node tools/empty-states.mjs`) |
| stacks / valores crus no renderer | **0 / 0** (`node tools/error-messages.mjs`) |
| tempo de abertura do app | **não medido** |

## O que falta, e o que trava cada item

### Depende do dono (nada a fazer sem isso)

- **CI que não cria runs.** Nenhuma run desde `41252692797`. Sem isso, nenhum item pode ser `VERIFIED`; todo merge de
  `host/**` ou `apps/**` fica com verificação local e desvio declarado. Ação: olhar a página Actions do repositório ou o
  GitHub Status.
- **Elenco dos mascotes (A ou B)** e **tap do fork** (`DZ23-LTDA/homebrew-codync`), para fechar o item 2 de verdade.
- **Xcode + Android SDK 37** para iOS/Android — `BLOCKED_BY_EXTERNAL_DEPENDENCY`.

### Depende de app aberto (inspeção visual — nenhuma feita nesta série)

- Item 4: instalação limpa, atualização e rollback nos três sistemas.
- Item 5: primeiro uso em menos de 3 minutos, QR, reconexão, desinstalação.
- Item 7: inspeção visual de cada tela em pt-BR (a cobertura de `tr()` não é cobertura de interface).
- Item 6: as 36 telas sem estado vazio, uma por uma, **e** a leitura das frases de erro (o scanner prova que nada vaza;
  não prova que a frase ajuda).

### Depende só de código (o que consigo fazer sem CI e sem a janela)

- Item 3: registrar que **downgrade não é suportado** e testar recuperação a partir de um backup real.
- Item 6 (parte): as 36 telas. **Cada uma é uma mudança de UI** — pelo próprio goal, exige inspeção; por isso não as
  estou alterando às cegas.
- Item 8: baseline de performance — precisa do app empacotado e de hardware descrito.

## Regra que me impus aqui

Não altero UI sem poder olhar a tela. As 36 telas ficam listadas até haver uma sessão com o app aberto; alterá-las às
cegas cumpriria o número e quebraria a única coisa que o gate realmente pede, que é a tela funcionar.

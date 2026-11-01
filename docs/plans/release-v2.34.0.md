# Release v2.34.0 — publicada e **não** instalada

Registro de 31/10/2026, 23:2xZ. O adendo v3.4 (regra 0) diz que PR não é progresso até estar instalado e usado. Este
documento diz exatamente onde essa frase está hoje.

## Feito

| Passo | Estado | Evidência |
|---|---|---|
| Versão igual nas três pontas | **2.34.0** | `apps/project.yml` (MARKETING_VERSION), `host/Cargo.toml`, `apps/desktop/package.json` |
| Instalador Windows construído | **sim** | `apps/desktop/dist/codync-windows-x64.exe`, 128,9 MB (NSIS, x64) |
| Release publicada no fork | **sim** | https://github.com/DZ23-LTDA/Codync/releases/tag/v2.34.0 — marcada *Latest*, 1 asset |
| Instalada e usada na máquina | **NÃO** | app instalado continua **2.33.2** — ver abaixo |

## O que travou a instalação

O instalador roda com `/S` e **não termina** neste ambiente: ficou mais de cinco minutos sem sair, com o app antigo já
encerrado e o host parado. Duas tentativas, o mesmo comportamento. Suspeitas, em ordem:

1. o NSIS `oneClick` tenta elevar e a janela de UAC não aparece neste contexto não interativo;
2. algum arquivo em `%LOCALAPPDATA%\Programs\codync-desktop` continua em uso por processo que eu não vi.

Nenhuma das duas foi confirmada — e por isso não são conclusão. O que foi **testado** depois:

| Teste | Resultado |
|---|---|
| Nenhum processo Codync vivo antes de instalar | confirmado (`Get-Process` vazio) |
| Instalador é `oneClick: true`, `perMachine: false` (instalação por usuário, sem UAC em geral) | confirmado em `electron-builder.yml` |
| `TMP`/`TEMP` dentro do workspace antes de rodar | **também travou** — a hipótese do diretório temporário caiu |
| Terceira tentativa, com limite de 150 s | não retornou; processo teve de ser morto |
| Versão instalada depois de tudo | **2.33.2 continua** — os arquivos do app têm data de 30/10, nada foi escrito |

Conclusão possível hoje: **o instalador NSIS não executa até o fim dentro deste ambiente não interativo**, e a causa não
está em UAC (é per-user), nem em processo vivo, nem em `TEMP`. Fica como bloqueio de ambiente, não como defeito do
produto.

## O que isso vale para o gate

- Item 10 ("ciclo de entrega funcionando") **subiu** de `NOT_STARTED` para `IN_PROGRESS`: existe release publicada com
  artefato e versão exibível. **Não** está `TESTED`, porque o aceite pede a atualização aplicada e o app instalado.
- A métrica do adendo (*tempo entre merge em `dev` e app instalado atualizado*) continua **infinita**: release às 02:10Z,
  instalado ainda em 2.33.2.
- O update in-app **não foi testado**.

## Próximo passo para fechar o item 10 nesta máquina

1. Instalar manualmente (duplo clique) — é o mesmo binário publicado; se a UAC aparecer para uma pessoa, o caminho existe.
2. Reabrir e conferir a versão em Sobre/Configurações.
3. Verificar que `%USERPROFILE%\.codync` sobreviveu.

Sem isso, "entregue" continua sendo a palavra que este adendo proibiu.

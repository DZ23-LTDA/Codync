# Fork da DZ23

Este repositório é um fork de [leepokai/Codync](https://github.com/leepokai/Codync) (Apache-2.0) mantido pela DZ23
para uso interno. Ele parte do `dev` do upstream e não é redistribuído como produto: o nome e o ícone continuam os do
projeto original, como o [TRADEMARK.md](TRADEMARK.md) exige de quem não redistribui.

## O que já difere do upstream

| PR | O que muda | Por quê |
|---|---|---|
| [#1](https://github.com/DZ23-LTDA/Codync/pull/1) | cada harness declara no catálogo o adapter que fala e se a memória própria pode ser desligada | tirar do código os `match backend` que cada harness novo obrigava a editar |
| [#2](https://github.com/DZ23-LTDA/Codync/pull/2) | sessão reabre com `session/resume` quando o agente não tem `loadSession` | o DeepSeek Harness anuncia só `resume`; sem isso ele perdia o contexto a cada restart |
| [#3](https://github.com/DZ23-LTDA/Codync/pull/3) | orçamento diário de tokens por bot, com gate antes do turno | limite de custo sem depender de plano ou provedor |
| [#5](https://github.com/DZ23-LTDA/Codync/pull/5) | contrato do `run` (uma tarefa, vários bots, um resultado) | base do próximo item: fan-out, join determinístico, ledger e selo |

O roadmap completo e o que ainda falta: `docs/plans/run-orchestration.md`.

## Como verificar

O CI do repositório roda em PR (`codync-host`): `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`
e `cargo nextest run --workspace --profile ci` em ubuntu, macOS e windows. O perfil `ci` (em
`host/.config/nextest.toml`) dá duas retentativas e corta um teste pendurado em cinco minutos; a suíte `network_e2e` é
pulada no windows, onde não alcança os pares.

Localmente, com o MSVC instalado, dá para rodar o mesmo em segundos:

```bash
cd host
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --bin codync-host chat::run   # um módulo; a suíte inteira precisa de Developer Mode
```

## Compilando no Windows

Pré-requisitos: Rust pela toolchain fixada em `host/rust-toolchain.toml` e **Visual Studio Build Tools** com a carga
"Desenvolvimento para desktop com C++" (linker, SDK do Windows e CMake). Sem o `link.exe` e o SDK, `cargo build` falha
no link e o `rusqlite` nem chega a compilar o SQLite embutido. O `aws-lc-rs` (usado por `codync-network`) também exige
NASM no Windows.

```powershell
cd host
cargo build
cargo run -- serve
```

## Onde estão as decisões

- `docs/plans/run-orchestration.md` — contrato do `run`.
- `docs/architecture/` — o que é do host e o que é de cada cliente.
- `docs/features/context-and-memory.md` — instruções, memória e compactação, por harness.

## Regras herdadas que valem para todo PR aqui

- O host Rust decide; clientes só renderizam (`docs/architecture/overview.md`).
- Campo novo no wire de bot exige registrar em `host/tests/fixtures/wire-shape.json`, senão o teste de contrato barra.
- `cargo clippy -D warnings` inclui dead code: um módulo novo precisa de consumidor real no mesmo PR.

## Skills de fora

`tools/import-skills.py` traz skills de repositórios públicos para `~/.codync/skills/<id>` (o mesmo lugar e o mesmo formato que o Codync usa: uma pasta com `SKILL.md`). Ele percorre o tarball, acha toda pasta com `SKILL.md`, copia e **pula id que já existe**, para nunca sobrescrever uma skill editada à mão.

```powershell
python tools/import-skills.py morluto/rea pbakaus/impeccable blader/humanizer
```

MCPs (Composio e conectores) são outro caminho: eles entram como servidores MCP do bot, com a chave no vault — não por este script.

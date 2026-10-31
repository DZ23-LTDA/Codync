# O2-B: por que não há reserva atômica hoje (e quando vai precisar)

O critério da Onda 2 pede **reserva/contabilização atômica antes de execuções concorrentes** para não estourar o
orçamento por *race condition*. Este documento registra a análise e a decisão: no modelo atual, essa corrida não
existe, e criar a reserva agora seria maquinário sem consumidor.

## O que é garantido hoje

- O orçamento é **por bot por dia** (`spend.<bot>.<dia>` em `kv`, inteiro em micro-dólares).
- Cada bot tem **um ator** que serializa turnos: o host enfileira mensagens e executa uma por vez (`agent/bot/queue.rs`).
  Dois turnos do mesmo bot nunca rodam ao mesmo tempo, então duas verificações de limite nunca ficam entre si.
- O gate roda no início do turno, lê o gasto corrente e recusa se já estiver no teto (`start_turn`). O gasto é somado
  no fim do turno, com o que o agente reportou.
- Logo, o pior caso de um único host é: um turno começa abaixo do teto e termina acima dele — **overshoot de um turno**,
  não de N turnos concorrentes. O teto se aplica ao próximo turno.

## Quando a corrida passa a existir

1. **Orçamento compartilhado por equipe, usuário ou tenant** (pilar 4.10): aí dois bots diferentes consomem o mesmo
   saldo e dois turnos concorrentes podem passar pelo mesmo check.
2. **Mais de um processo de host** sobre o mesmo banco.
3. **Orçamento por run** com fan-out paralelo, quando o run dispara N bots de uma vez.

## O desenho para o dia em que for necessário

- Reserva no `kv` como **compare-and-swap** por chave de saldo: ler `spent`, calcular `spent + reserva`, gravar só se a
  revisão não mudou (`kv_set_stamped`), e liberar a reserva no fim do turno devolvendo a diferença entre reservado e real.
- Reserva dimensionada pelo **custo estimado do turno** (última média do bot × preço declarado), não por um valor fixo.
- Reserva órfã expira por *lease* (timestamp), para um host que morreu no meio não congelar o saldo.
- Teste obrigatório: dois turnos concorrentes no limite, exatamente um passa.

## Decisão

Não implementar reserva atômica até que o orçamento deixe de ser por bot. Registrar o critério aqui e no checkpoint.
Quando o primeiro dos três casos acima entrar, este documento é a especificação do PR.

# Mascotes (B-3PO)

Um lugar só para a arte, e derivação por script para cada cliente. Quatro cópias mantidas à mão em quatro pastas é como
os mascotes ficam diferentes entre si.

## Contrato

- Caminho canônico: **`assets/mascots/<avatarId>.png`**.
- `avatarId` é exatamente um dos 13 ids de `host/src/agent/avatar.rs` (`IDS`).
- 256 a 512 px no lado maior, PNG otimizado, fundo transparente.
- **Sem nome e sem cargo embutidos no arquivo** (goal §5): a identidade visual é só a imagem.
- Enquanto um PNG não existe, o host valida e guarda o id do mesmo jeito e o cliente mostra o fallback
  (`byte`, hoje). Faltar arte não quebra nada.

## Verificar e derivar

```bash
node tools/mascot-assets.mjs            # o que existe, o que falta, o que está fora do contrato
node tools/mascot-assets.mjs --derive   # escreve as cópias de cada cliente a partir da fonte canônica
```

O script lê os 13 ids do próprio código do host, então a lista nunca divergir da realidade: acrescentar um mascote é
acrescentar o id em `avatar.rs` e o PNG aqui.

## Licença

Ver `NOTICE` nesta pasta: a arte é ativo de marca da DZ23, **não** licenciada para reuso; o código do repositório segue
em Apache-2.0.

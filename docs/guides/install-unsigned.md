# Distribuição assistida (não assinada)

O B-3PO é distribuído hoje **sem certificado**: os instaladores não são assinados pelo Windows nem pelo macOS. Não é
falta de trabalho nem arquivo corrompido — é uma decisão do projeto enquanto não existir certificado de code signing.

Este documento existe para que instalar não seja um mistério. Os três avisos do sistema são esperados, e todos têm um
caminho explícito.

## macOS

O app não é assinado, então o Gatekeeper bloqueia a primeira abertura com "não é possível abrir porque o desenvolvedor não
pode ser verificado" ou "está danificado".

1. Arraste `Codync.app` para `/Applications`.
2. Abra o Terminal **uma vez** e rode:

   ```bash
   xattr -cr /Applications/Codync.app
   ```

3. Abra o app normalmente (clique com o botão direito → Abrir também funciona).

**"Está danificado" não significa arquivo corrompido.** O macOS usa essa frase quando o binário não tem assinatura válida
para a política atual. `xattr -cr` remove o atributo de quarentena do download; o app não é modificado.

Artefatos não assinados saem do CI com o sufixo `-unsigned` no nome, para nunca serem confundidos com uma build assinada.

## Windows

O SmartScreen mostra "O Windows protegeu o computador".

1. Clique em **Mais informações**.
2. Clique em **Executar assim mesmo**.

O instalador é o mesmo que um assinado; só não há assinatura para o Windows verificar.

## Linux

O AppImage não tem bit de execução depois de baixar:

```bash
chmod +x Codync-*.AppImage
./Codync-*.AppImage
```

Se a distribuição usar FUSE e ele não estiver instalado: `./Codync-*.AppImage --appimage-extract-and-run`.

## O que ainda depende do dono

- Certificado Apple (Developer ID + notarização): assinar e notarizar exige conta Apple e certificado válidos. Enquanto
  não existir, o job `macos` do `release-desktop.yml` empacota sem assinatura e sem notarização.
- Certificado Windows (Authenticode): mesma situação para o SmartScreen.
- Com o certificado configurado (`DESKTOP_MACOS_SIGNING=true` e os segredos `APPLE_*`/`ASC_*`), o mesmo job assina,
  notariza e volta a gerar o feed de atualização; o caminho assinado não foi removido.

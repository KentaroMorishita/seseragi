# Seseragi editor support

`seseragi/`は現行の正式なSeseragi VS Code extensionです。Marketplace IDは
`seseragi-dev.seseragi`です。`seseragi-legacy/`は旧Marketplace IDを更新するためだけの
migration stubで、LSPをbundle・起動しません。

すべての`.ssrg`とuntitled documentへTextMate grammarを適用し、同梱した
`crates/seseragi-lsp`からhover、completion、signature help、definition、diagnostic、
quick fix、semantic tokensを提供します。

## Build and package

```sh
cd extensions/seseragi
bun install --frozen-lockfile
bun run package
```

package scriptは現在のplatform用`seseragi-lsp`をrelease buildし、VSIXへ一つだけ同梱し、
manifest、license、server target、package sizeを検査します。出力先は
`target/seseragi-v<version>-vscode-<platform>.vsix`です。

macOS arm64/x64、Linux x64、Windows x64のpackageは
`.github/workflows/vscode-extension.yml`で個別に生成します。公式releaseでは
`seseragi-v<version>-vscode-<platform>.vsix`として添付され、release tagはtoolchain
versionと一致する`v<version>`を使います。

独自serverを試す場合だけ`seseragi.languageServer.path`を設定してください。
旧formatter IDの置換と旧extensionのdisable / uninstallは
[`seseragi/README.md`](./seseragi/README.md)のmigration手順を参照してください。

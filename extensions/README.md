# Seseragi editor support

`seseragi-spec-preview/`は現行の正式なSeseragi VS Code extensionです。
directory名とpackage名、extension ID `seseragi-dev.seseragi-spec-preview`だけは
旧0.1.0を上書き更新し、二重起動を防ぐために維持しています。syntax-onlyの旧実装は
残っていません。

すべての`.ssrg`とuntitled documentへTextMate grammarを適用し、同梱した
`crates/seseragi-lsp`からhover、completion、signature help、definition、diagnostic、
quick fix、semantic tokensを提供します。

## Build and package

```sh
cd extensions/seseragi-spec-preview
bun install --frozen-lockfile
bun run package
```

package scriptは現在のplatform用`seseragi-lsp`をrelease buildし、VSIXへ一つだけ同梱し、
manifest、license、server target、package sizeを検査します。出力先は
`target/seseragi-vscode-<platform>.vsix`です。

macOS arm64/x64、Linux x64、Windows x64のpackageは
`.github/workflows/vscode-extension.yml`で個別に生成します。release tagはextension versionと
一致する`vscode-v<version>`を使います。

独自serverを試す場合だけ`seseragi.languageServer.path`を設定してください。

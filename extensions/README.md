# Seseragi editor support

このdirectoryでは`seseragi-spec-preview`を現行Seseragi VS Code extensionとして
維持します。既存installのupgradeを保つためextension IDは
`seseragi-dev.seseragi-spec-preview`のまま、language IDは`seseragi`へ統一しています。
すべての`.ssrg`へTextMate highlightingとnative LSPの
hover、completion、signature help、definition、quick fix、semantic tokenを提供します。

Rust compilerのparser、型検査、diagnosticはextensionへ複製しません。薄いclientが
`crates/seseragi-lsp`のstdio serverを起動し、Playgroundと同じshared Analysis APIを使います。

## Local install

```sh
cargo install --path crates/seseragi-lsp
cd extensions/seseragi-spec-preview
bun install --frozen-lockfile
bun run package
code --install-extension ../../target/seseragi-vscode.vsix --force
```

repository内のdebug binaryを使う場合はVS Code setting
`seseragi.languageServer.path`をabsoluteな`target/debug/seseragi-lsp`へ変更してください。

## License

Apache-2.0

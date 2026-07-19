# Seseragi editor support

このdirectoryで現在維持しているのは
`seseragi-spec-preview`のsyntax-only VS Code extensionだけです。
`examples/spec/**/*.ssrg`のTextMate highlightingと、canonical exampleを検査する
scope contractを提供します。

Rust compilerのparser、型検査、diagnostic、formatterはextensionへ複製しません。
native LSPは`crates/seseragi-lsp`にあり、VS Code packagingとの接続は別sliceです。

## Local install

```sh
cd extensions/seseragi-spec-preview
bunx vsce package --out /private/tmp/seseragi-spec-preview.vsix
code --install-extension /private/tmp/seseragi-spec-preview.vsix --force
```

## License

Apache-2.0

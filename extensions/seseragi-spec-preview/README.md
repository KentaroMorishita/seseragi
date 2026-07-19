# Seseragi Spec Preview

canonical仕様と `examples/spec/**/*.ssrg` 専用のsyntax-only VS Code extensionです。
Rust compilerとLSPの実装は含まず、TextMate grammarだけを提供します。

拡張自身のconfiguration defaultにより `examples/spec/**/*.ssrg` だけを
`seseragi-spec-preview` language idへ割り当てます。

## Local install

```sh
cd extensions/seseragi-spec-preview
bunx vsce package --out /private/tmp/seseragi-spec-preview.vsix
code --install-extension /private/tmp/seseragi-spec-preview.vsix --force
```

インストール後にVS Code windowをreloadしてください。

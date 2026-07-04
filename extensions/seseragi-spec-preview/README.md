# Seseragi Spec Preview

刷新中の仕様と `examples/spec/*.ssrg` 専用のsyntax-only VS Code extensionです。
現行compiler、LSP、既存のSeseragi拡張から独立しています。

このworkspaceでは `.vscode/settings.json` により `examples/spec/*.ssrg` だけを
`seseragi-spec-preview` language idへ割り当てます。

## Local install

```sh
cd extensions/seseragi-spec-preview
bunx vsce package --out /private/tmp/seseragi-spec-preview.vsix
code --install-extension /private/tmp/seseragi-spec-preview.vsix --force
```

インストール後にVS Code windowをreloadしてください。

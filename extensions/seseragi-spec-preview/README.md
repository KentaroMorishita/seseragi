# Seseragi for VS Code

`.ssrg`のTextMate highlightingと、現行Rust compilerのnative language serverを提供します。
hover、completion、signature help、definition、quick fix、semantic tokenはPlaygroundと
同じshared Analysis APIから得られます。

extensionにcompilerロジックは複製せず、PATH上の`seseragi-lsp`をstdioで起動します。
別のbinaryを使う場合は`seseragi.languageServer.path`を設定してください。

## Local install

```sh
cargo install --path ../../crates/seseragi-lsp
cd extensions/seseragi-spec-preview
bun install --frozen-lockfile
bun run package
code --install-extension ../../target/seseragi-vscode.vsix --force
```

インストール後にVS Code windowをreloadしてください。

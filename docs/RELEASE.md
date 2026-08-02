# Release contract

Seseragiの公式toolchain versionの唯一のsourceは、root
[`Cargo.toml`](../Cargo.toml) の`[workspace.package].version`です。すべてのRust
crateは`version.workspace = true`を使い、CLI、LSP、runtime、WASM、Playground、VS
Code extensionのpackage versionは`bun run release:sync`でこの値へ同期します。

現在の対応関係は、次のmachine-readable commandで確認できます。

```sh
bun run release:info
cargo run -p seseragi-cli -- --version
cargo run -p seseragi-lsp -- --version-json
```

`seseragi --version`はtoolchain version、commit、build channel、target、dirty状態を
表示します。`seseragi-lsp --version-json`は同じmetadataに加えてprotocol / analysis
schema versionを返します。VS Code extensionは起動前とinitialize後の両方で、同梱LSPの
toolchain version、protocol、analysis schemaをmanifestと照合します。

## Version bump

新しいversionを作るときは、複数manifestを手で書き換えません。

```sh
bun run release:bump -- 0.4.1
# CHANGELOG.mdへ0.4.1のentryを追加
bun run build:playground:wasm
bun run release:check
```

`release:bump`はcanonicalなCargo workspace versionと派生JavaScript manifestだけを
更新します。WASM packageはRust crate metadataから生成されるため、必ずgeneratorを通します。
`release:check`はCargo manifest / lock、runtime、Playground、WASM、extension、CHANGELOG
のversion driftを拒否し、CIも同じgateを実行します。

## Channelとtag

正式releaseはcleanな`v<version>` tagでbuildした場合だけです。tagとversionが一致しない
build、未tag commit、またはdirty worktreeはすべて`development` channelとしてmetadataへ
記録され、正式releaseを名乗りません。build metadataのcommit、target、dirty状態は利用者が
local buildと公開artifactを区別するための情報であり、version sourceを上書きしません。

release tagは`v0.4.0`のようにtoolchain versionと同じ名前を使います。旧
`vscode-v0.3.0` tagは履歴であり、新しいreleaseに使用しません。

tag workflowは次を同じversionで生成します。

- `seseragi-v<version>-<target>` (CLI)
- `seseragi-lsp-v<version>-<target>` (LSP)
- `seseragi-v<version>-vscode-<target>.vsix`
- `seseragi-legacy-migration-v<version>.vsix`（旧extension IDを更新する非LSP stub）
- `seseragi-runtime-v<version>.tar.gz`
- `seseragi-wasm-v<version>.tar.gz`

GitHub Releaseの本文はroot `CHANGELOG.md`の該当entryから`bun run release:notes`で生成します。

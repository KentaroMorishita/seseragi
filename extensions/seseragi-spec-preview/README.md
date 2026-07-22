# Seseragi for VS Code

Seseragiの正式なVS Code extensionです。保存場所に関係なく、すべての`.ssrg`と
untitled documentへ次を提供します。

- TextMateによる起動直後のsyntax highlighting
- 現行Rust compilerと同じAnalysis APIを使うnative language server
- hover、completion、signature help、definition、diagnostic、quick fix
- semantic tokensによる型・symbol情報を使ったhighlightの補完

TextMate grammarはserver起動前も使える字句highlightを担当し、semantic tokensは
その色付けを型情報で上書き・補完します。compilerロジックはextensionへ複製しません。

## Install

GitHub ReleaseからOSとCPUに合うVSIXを取得し、VS Codeで
`Extensions: Install from VSIX...`を実行します。CLIからは次のように更新できます。

```sh
code --install-extension seseragi-vscode-darwin-arm64.vsix --force
```

VSIXには対応する`seseragi-lsp`が一つだけ同梱されるため、通常利用で
`cargo install`やPATH設定は不要です。

対応package:

- macOS arm64 / x64
- Linux x64
- Windows x64

## Language Server

status barの`Seseragi`からserver状態と専用Output Channelを確認できます。
Command Paletteには次があります。

- `Seseragi: Restart Language Server`
- `Seseragi: Show Language Server Output`

独自buildを使う場合だけ、`seseragi.languageServer.path`へabsolute pathまたは
PATH上のcommandを設定してください。起動時にbinary version、protocol version、
analysis schema versionを検査し、互換性がない場合はコンパイル機能を開始しません。

## Upgrade from 0.1.0

syntax-onlyだった`Seseragi Spec Preview 0.1.0`と同じextension ID
`seseragi-dev.seseragi-spec-preview`を維持しています。新しいVSIXを`--force`付きで
installすれば既存extensionを上書きでき、二つのclientが同時に起動しません。

package名とrepository directoryの`seseragi-spec-preview`はupgrade identityを守るためだけに
残しています。表示名、language ID、公開機能は正式な`Seseragi`へ統一済みです。

## Development

repository rootから次を実行すると、現在のplatform用serverをrelease buildし、
VSIXの内容まで検証します。

```sh
cd extensions/seseragi-spec-preview
bun install --frozen-lockfile
bun run package
```

出力は`target/seseragi-vscode-<platform>.vsix`です。

## License

Apache-2.0

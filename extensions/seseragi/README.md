<p align="center">
  <img src="../../assets/brand/source/seseragi-icon.svg" alt="Seseragi symbol" width="180">
</p>

# Seseragi for VS Code

Seseragiの正式なVS Code extensionです。保存場所に関係なく、すべての`.ssrg`と
untitled documentへ次を提供します。

- TextMateによる起動直後のsyntax highlighting
- 現行Rust compilerと同じAnalysis APIを使うnative language server
- hover、completion、signature help、definition、diagnostic、quick fix
- CLIと同じformatterを使うFormat Document
- semantic tokensによる型・symbol情報を使ったhighlightの補完

TextMate grammarはserver起動前も使える字句highlightを担当し、semantic tokensは
その色付けを型情報で上書き・補完します。compilerロジックはextensionへ複製しません。

## Install

GitHub ReleaseからOSとCPUに合うVSIXを取得し、VS Codeで
`Extensions: Install from VSIX...`を実行します。CLIからは次のように更新できます。

```sh
code --install-extension seseragi-v0.4.6-vscode-darwin-arm64.vsix --force
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

workspace folder内では、開いている`.ssrg`から到達可能なrelative importを同じproject
resolverで追跡します。`seseragi.toml`があるfolderではそのmanifest、path dependency、
package importを使い、保存前のopen bufferも解析に反映します。source fileまたは
`seseragi.toml`の作成・削除・rename時には、影響するmodule graphのdiagnosticを更新します。
workspace外とuntitled documentはsingle-file解析へ安全にfallbackします。

## Formatting

`.ssrg`では、このextensionがdefault formatterとして選択されます。Command Paletteの
`Format Document`またはeditorの`Format Document With...`から、未保存のdocumentも
CLIの`seseragi format`と同じ規則で整形できます。

保存時にも整形する場合は、VS Codeの設定へ次を追加します。

```json
{
  "[seseragi]": {
    "editor.defaultFormatter": "seseragi-dev.seseragi",
    "editor.formatOnSave": true
  }
}
```

syntax errorがあるdocumentは書き換えず、language serverが通常のdiagnosticを表示します。
range formattingとon-type formattingは提供しません。

## Migration from the preview ID

正式extension IDは`seseragi-dev.seseragi`、package名は`seseragi`です。旧
`seseragi-dev.seseragi-spec-preview`は正式IDではなく、更新時にLSPを起動しない
migration stubへ置き換わります。

1. OSとCPUに合う`seseragi-v<version>-vscode-<platform>.vsix`をinstallします。
2. 旧extensionが残る場合は、先に`seseragi-legacy-migration-v<version>.vsix`で更新するか、
   VS CodeのExtensions viewから旧extensionをdisable / uninstallします。
3. `Seseragi: Migrate Legacy Settings`を実行して、`[seseragi]`の旧formatter IDを新IDへ
   置き換えます。明示的に設定していない場合は変更しません。

旧extensionが0.3系のままactiveなら、正式extensionは二つ目のLSPを起動せずmigrationを
案内します。`seseragi.languageServer.path`のnamespaceと値は変わらないため、custom LSP
pathを設定済みでも再入力は不要です。

## Development

repository rootから次を実行すると、現在のplatform用serverをrelease buildし、
VSIXの内容・archive内の実行mode・一時展開後の`--version-json`まで検証します。

```sh
cd extensions/seseragi
bun install --frozen-lockfile
bun run package
```

出力は`target/seseragi-v<version>-vscode-<platform>.vsix`です。

CIはmacOS arm64 / x64、Linux x64、Windows x64で同じpackage / extract /
execution smokeを実行します。tag releaseではupload前のrelease artifactにも同じ検証を
再実行します。起動失敗時はSeseragi Language Server Outputに、選択したbinary pathと
missing / permission / target / versionの原因を表示します。

このgateはVSIX内のnative binaryの配置・mode・実行契約を対象にします。macOSの
quarantine、code signing、notarizationは別のrelease signing policyであり、手動`chmod`を
利用者向け回避策にはしません。OSが署名・quarantineを理由に起動を拒否した場合も、Outputの
実行pathとOSエラーをもとに配布artifactを確認します。

## License

Apache-2.0

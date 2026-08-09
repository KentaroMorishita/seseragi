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

tagは必ず最新`main`へ統合済みのrelease commitへ付けます。tag workflowはtagが指す
commitとcheckoutしたcommitが同じこと、そのcommitが実行時点の`origin/main`履歴へ
含まれることを最初に検証します。その後、Rust workspace、canonical conformance、native
sample / Tour、WASM freshness、Playground test / typecheck / production build、extension
contractを`bun run check:release-gate`で検証します。いずれかが失敗した場合、artifact buildと
GitHub Release publishは開始されません。

```sh
git switch main
git pull --ff-only origin main
bun run check:release
git tag -a v0.4.0 -m "Seseragi v0.4.0"
git push origin v0.4.0
```

tag workflowは次を同じversionで生成します。

- `seseragi-v<version>-darwin-arm64.tar.gz` (macOS Apple Silicon CLI / LSP)
- `seseragi-v<version>-darwin-x64.tar.gz` (macOS Intel CLI / LSP)
- `seseragi-v<version>-linux-x64.tar.gz` (Linux x64 CLI / LSP)
- `seseragi-v<version>-win32-x64.zip` (Windows x64 CLI / LSP)
- 各native archiveと同名の`.sha256`
- `seseragi-v<version>-vscode-<target>.vsix`
- `seseragi-legacy-migration-v<version>.vsix`（旧extension IDを更新する非LSP stub）
- `seseragi-runtime-v<version>.tar.gz`
- `seseragi-wasm-v<version>.tar.gz`

GitHub Releaseの本文はroot `CHANGELOG.md`の該当entryから`bun run release:notes`で生成します。

native archiveの直下には`seseragi`と`seseragi-lsp`（Windowsでは`.exe`付き）だけを
収録します。macOS / Linuxの2 binaryはmode `755`です。tag workflowはarchive作成前だけでなく、
Actions artifactから再downloadした後にもchecksum、収録file、mode、version、targetを確認し、
両binaryを展開直後のpathから実行します。GitHub Releaseへ添付するのは、この再検証済みの
archiveとchecksumです。

release gateが確定したcommit SHAは、全artifact jobのcheckoutとActions artifact名へ
埋め込みます。publish jobは同じworkflow run内でそのSHAを含むartifact名だけをdownloadし、
native archiveのdownload後smoke、全VSIX package、WASM / runtime archiveが成功した場合だけ
開始します。job retryでは同じSHAのartifactだけを置換・再利用するため、別commitの成果物が
混在しません。source full gateではnative / VSIXをpackageせず、実際にuploadするmatrix jobが
一度だけ生成して検証します。WASM archiveはsource gateがfreshnessを確認した同一SHAの
committed packageを再buildせずに収録します。

## Release failureからの復旧

一時的なrunner障害で、tag commitと`main`が変わっていない場合は、同じworkflow runの
failed jobだけをretryします。SHA付きartifact名が同じcommitへ固定されるため、成功済みjobの
artifactと安全に合流できます。

main包含またはfull gateが失敗した場合はtagを強制移動せず、修正を通常branchから`main`へ
統合します。GitHub Releaseがまだ作られていないことを確認して失敗tagを削除し、修正commitへ
同名tagを作り直します。

```sh
git push origin :refs/tags/v0.4.0
git tag -d v0.4.0
git switch main
git pull --ff-only origin main
git tag -a v0.4.0 -m "Seseragi v0.4.0"
git push origin v0.4.0
```

publish job自体が失敗して不完全なGitHub Releaseが作られた場合は、添付assetとtarget SHAを
確認し、不完全なreleaseを削除してから同一SHAのpublish jobをretryします。公開済みreleaseの
tagを別SHAへ付け替えません。

## Native archiveからinstall

macOS / Linuxでは、GitHub ReleaseからOS / CPUに合うarchiveとchecksumを取得します。

```sh
version=0.4.0
target=darwin-arm64 # darwin-x64 または linux-x64
archive="seseragi-v${version}-${target}.tar.gz"
curl -LO "https://github.com/KentaroMorishita/seseragi/releases/download/v${version}/${archive}"
curl -LO "https://github.com/KentaroMorishita/seseragi/releases/download/v${version}/${archive}.sha256"
# macOS: shasum -a 256 -c "${archive}.sha256"
# Linux: sha256sum -c "${archive}.sha256"
tar -xzf "$archive"
./seseragi --version
./seseragi-lsp --version-json
```

Windows x64では`seseragi-v<version>-win32-x64.zip`と同名の`.sha256`を取得し、
`Get-FileHash -Algorithm SHA256`の値がchecksum fileの先頭値と一致することを確認してから
`Expand-Archive`します。展開先には`seseragi.exe`と`seseragi-lsp.exe`だけが含まれます。

```powershell
$version = "0.4.0"
$archive = "seseragi-v$version-win32-x64.zip"
$base = "https://github.com/KentaroMorishita/seseragi/releases/download/v$version"
Invoke-WebRequest "$base/$archive" -OutFile $archive
Invoke-WebRequest "$base/$archive.sha256" -OutFile "$archive.sha256"
$expected = (Get-Content "$archive.sha256").Split()[0]
$actual = (Get-FileHash $archive -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -ne $expected) { throw "SHA-256 checksum mismatch" }
Expand-Archive $archive
& ".\\$($archive.Replace('.zip', ''))\\seseragi.exe" --version
```

# Scoped checks

Seseragiの検証は、変更範囲に対応するscoped laneを先に実行します。`bun run check`
は、各laneを日常作業で無条件に繰り返すためのコマンドではなく、repository-wideな
最終統合gateです。

## Lane matrix

| 変更範囲 | コマンド | 含まれる検証 |
| --- | --- | --- |
| sample metadata / source | `bun run check:sample` | native CLI sample実行、全sampleのWASM compile / format、Playground sample manifestのfreshness |
| sample / Playground UI | `bun run check:playground` | sample基盤、Playground / catalog lint、Tour manifest、Playground test、TypeScript typecheck、Vite build |
| Web UI browser review | `cd apps/playground && bun run test:visual` | Chromiumで全HTML sampleのviewport / interaction / image fallback / Code / Explorerを確認し、review PNGとHTML reportを出力 |
| Rust / compiler / LSP workspace | `bun run check:rust` | Rust format、workspace test（対象crateだけなら `bun run check:rust -- -p <crate>`） |
| conformance fixture | `bun run check:conformance` | canonical conformance runner（対象rootを引数で限定可能） |
| compiler/runtime/WASM boundary | `bun run check:wasm` | committed Playground WASMの再生成と差分確認 |
| VS Code extension | `bun run check:extension` | official ID / legacy migration boundary、extension lint・test、host向け正式VSIXのarchive mode・展開・`--version-json` smoke、非LSP migration VSIXのpackage / verify |
| release metadata / artifact naming | `bun run check:release` | canonical version source、Cargo/JS/WASM version同期、CHANGELOG、release contract script |
| repository-wide | `bun run check` または `bun run check:full` | format、lint、Rust workspace、全conformance、native samples、WASM、Playground、extension |

各laneは `scripts/check-scoped.sh` を共通runnerとして使います。Playgroundのtest、
typecheck、buildはcatalog / Tour manifest確認をlane内で一度だけ済ませてから直接実行し、
`package.json`の個別scriptが同じcatalog確認を三回繰り返す構造を避けています。

`check:sample`はCLI実行対象だけでなくbrowser-interactive sampleもcommitted WASMで
project compileし、全sourceがformatterのcanonical outputと一致することを確認します。
manifest/hashだけが更新され、壊れたinteractive sourceが通過する状態にはしません。

## Dependency installation

通常のscoped checkは依存installを行わず、lockfileで管理されたlocal binaryだけを使います。
必要なdependencyが無い場合は暗黙downloadへ進まず、bootstrap commandを表示して失敗します。
新しいworktreeやCI runnerでは、検証前に対象workspaceを一度bootstrapしてください。

```sh
bun install --frozen-lockfile
cd apps/playground && bun install --frozen-lockfile
cd ../../extensions/seseragi && bun install --frozen-lockfile
```

`check:sample`と`check:playground`はcommitted WASMまたは固定されたPlayground
TypeScript toolchainを使うため、Playground dependencyのbootstrapが必要です。
`check:conformance`はそれらに加えてruntime package probeがroot側のNode型を使うため、
rootとPlaygroundの両方をbootstrapしてください。lockfileまたは依存関係を変更した場合も、
該当workspaceでfrozen installを明示的に行います。full gateはrootとPlaygroundを先に
bootstrapし、extension packaging時にもfrozen installを行います。

## Web UI browser review

`apps/playground/tests/fixtures/web-ui-regression.json`が、全HTML sample、320px /
iPhone / Android / landscape / desktop、必要surface、interaction stateを定義します。
通常の`check:playground`はこのmatrixとsource readabilityをBun testで固定し、Chromiumを
暗黙downloadしません。実ブラウザを使うreview時だけ次を実行します。

```sh
cd apps/playground
bun run test:visual:install
bun run test:visual
```

Playwrightはfixed Unsplash URLをdeterministicなlocal SVGへrouteし、layout、overflow、
contrast、keyboard reachable control、image failure fallbackを確認します。成功時も
`test-results/web-ui-review/`へstate別PNGとHTML reportを残します。GitHub Actionsの
`web-ui-visual.yml`はこのdirectoryをartifactとしてuploadするため、browser差分はCI成功
だけでは見落とせません。

## Full gateを実行する条件

次のいずれかに該当するときだけ、scoped laneに加えて `bun run check` を実行します。

- compiler / runtime / WASM基盤を変更した
- 複数の領域を横断した
- release前の統合確認を行う
- #201のqueueをmainへ統合する
- Issue本文がfull checkを明示的に要求する

full gateを実行した場合は、作業ログまたはPR本文へ「必要理由」と「結果」を記録します。
長時間無出力になった場合は、該当phaseとprocess activityを確認し、放置しません。

## CIとlocalの責務

- localの末端Issue作業は、変更範囲に対応するscoped laneを担当する。
- `.github/workflows/vscode-extension.yml` はextension path変更時のOS別VSIX package、
  archive mode、展開後native LSPの`--version-json` smokeを担当する。local
  `check:extension`はhost platformの短い再現確認であり、tag releaseでも同じ検証器を
  upload前のrelease VSIXへ再実行する。
- `.github/workflows/release-contract.yml` はversion sourceやartifact namingの変更で
  `check:release`を実行する。tag releaseは`v<version>`だけを受け付ける。
- repository-wideのfull gateは、compiler/runtime/WASM変更、release、queue統合などの
  integration pointで明示的に実行する。CIに同じ長時間検査を各sample変更へ無条件に重ねない。
- scoped laneが不足している場合は、対象コマンドを個別に実行して結果を記録し、lane追加を
  次の基盤Issueとして扱う。

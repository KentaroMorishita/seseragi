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
| release metadata / native artifact | `bun run check:release` | canonical version source、Cargo/JS/WASM version同期、CHANGELOG、host向けCLI / LSP archive・checksum・再展開実行smoke |
| tag release source gate | `bun run check:release-gate` | repository-wide full gate相当。ただしnative / VSIXのpackage smokeは同一SHAを使うrelease matrix jobへ委譲し、成果物を二重生成しない |
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
contrast、keyboard reachable control、image failure fallbackを確認します。代表stateは
review済みscreenshot baselineとも比較し、失敗時は`test-results/web-ui-review/`へ
expected / actual / diff、trace、HTML reportを残します。OS固有のfont rasterizationを
layout差分と誤認しないようbaselineはplatform別に保持し、localとGitHub Actionsはともに
`bun run test:visual`をverify commandとして使います。

意図したUI変更でbaselineを更新するときは、先に
[`SHOWCASE_QUALITY.md`](./SHOWCASE_QUALITY.md)のhuman reviewを完了し、review理由を必須にした
次のコマンドを使います。PNGと承認済み`showcase-review.json`のSHA-256、理由は
`e2e/visual-baselines.review.json`へ記録され、PNGだけまたはapproval artifactだけの変更は
verify前に失敗します。

```sh
bun run test:visual:update -- "変更理由"
```

update commandはreview対象のbaselineを持つWeb UI regression specだけを再実行してPNGと
review hashを更新します。通常の`bun run test:visual`はguideを含む全Playwright specを
引き続き実行するため、baseline更新のたびに無関係なbrowser testを重ねず、最終verifyの
coverageは狭めません。

Linux baselineも同じupdate commandを使い、`Web UI visual regression`の
workflow_dispatchへ`update_reason`を渡して生成します。

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
  `check:release`を実行し、Linux native archiveも実binaryで検証する。tag releaseは
  `v<version>`だけを受け付け、tag commitのmain包含と`check:release-gate`成功後にだけ
  同一SHAの全platform artifactを生成し、upload前と再download後に検証する。
- repository-wideのfull gateは、compiler/runtime/WASM変更、release、queue統合などの
  integration pointで明示的に実行する。CIに同じ長時間検査を各sample変更へ無条件に重ねない。
- scoped laneが不足している場合は、対象コマンドを個別に実行して結果を記録し、lane追加を
  次の基盤Issueとして扱う。

# Scoped checks

Seseragiの検証は、変更範囲に対応するscoped laneを先に実行します。`bun run check`
は、各laneを日常作業で無条件に繰り返すためのコマンドではなく、repository-wideな
最終統合gateです。

## Lane matrix

native BigInt parse/format/fromIntはbackendの`--test basic_big_int`、runtimeの
`big_int_tests`、CLIの`--test basic_standard basic_big_int_`で確認します。
任意桁数をFloatや固定幅整数へ丸めず、符号・zeroの正規化、strict decimal構文と
最初の不正UTF-8 byte位置、canonical owner/kind/source stageを保ちます。
元raw TS両profileのstrictとfresh Bun/Node/native、通常path packageのopaque値・
保存formatter、Ref/cold再実行・Failure/Defect後のfinalizer/root0を比較します。
radix指定・BigInt算術と全辞書、Decimal、program WASMの新しい受入は含めません。

nativeのText split/trim/contains/replaceとMath sqrt/sin/cos/logは、backendとCLIの
`--test basic_standard`、runtimeの`math_tests`/`text_tests`で確認します。
Unicode scalar・White_Space・literal replacement、canonical owner/kind/source stageの拒否、
数学関数の特殊値と仕様の許容誤差、元のraw TS両profileのstrict、fresh Bun/Node/native、
saved functionのcold再実行とFailure/Defect後のfinalizer/root0、通常path packageの
CSV変換・不正入力時の旧出力保持を確認します。旧sqrt拒否sourceは成功対照へ保持し、
未対応cbrtとList comprehensionの位置付き拒否を残します。新8 APIのprogram WASM受入は
追加せず、target-specific proofを欠く入力を位置付きで拒否します。

native Bytes fromInts/toInts/get/toIntはbackendの`--test basic_bytes`、runtimeの
`bytes_tests`、CLIの`--test basic_standard basic_bytes_`で確認します。
全unsigned byte・空入力・最初の範囲外Int・負/大index・入力不変・独立storage、
canonical owner/kind/source stageと正確な位置、元raw TS両profileのstrict、
fresh Bun/Node/native、Refに保存したbufferと関数のcold再実行・Failure/Defect後の
finalizer/root0を保ちます。新しいprogram WASM ABIやByte/Bytesの全標準辞書、
残るBytes APIを受け入れたことにはしません。

native Array filter/find/get/length/take/dropはbackendの`--test basic_array`、runtimeの
`array_operation_tests`、CLIの`--test basic_standard basic_array_`で確認します。
genericな要素型・canonical owner/kind・保存/部分適用とsource stageを検証し、
元のraw TS両profileのstrict、fresh Bun/Node/nativeと通常path packageを比較します。
filterの順序、findの途中停止、空入力、負/大indexとcount、入力不変と独立storage、
関数要素のcapture・Ref cycle、cold再実行・Failure/Defect後のfinalizer/root0を保ちます。
receiver-firstな既存Array index ABIを変えず、新6 APIのprogram WASMは位置付きで拒否します。

native Int infix `>` / `>=`の修復はbackendの`--test comparison_partials`とCLIの
`--test basic_standard basic_array_infix_descending_`で確認します。
正式112で拒否された元のArray filterのsource、整数境界と真偽、canonical Ordと
同名binder・不正Coreの位置、既存の保存比較・user methodのstageを保ちます。
新しいinfix経路のprogram WASM受入は追加しません。

List prefixの共有はloweringの`--test match_decision`、backendの
`--test native list_prefix`、CLIの`--test rust_backend
list_prefix_decisions_and_effects_match_raw_strict_ts_and_native`で確認します。
元の生成TS両profileのstrictとfresh Bun/Node/nativeを比較し、guard失敗・長さ・
nested payload・alias・tail捕捉・Ref cycle・Failure解放を保ちます。lexicalな
`List` binderと未証明rootは認定しません。幅1/4/16/32はtrackedな
`measure_list_prefix.py`で測り、targetごとに改善と速度回帰を区別します。

閉じた標準tuple Show/Debug辞書のlazy reuseは、backendの`--test closed_display`と
CLIの`--test rust_backend closed_display_`で確認します。canonical owner/type/child/
arityとfactoryの純粋性を共有CoreIrで認定し、Parameter/user/generic/HKT/未証明factoryは
元の経路を保ちます。raw TS両profileのstrict、fresh Bun/Node/native、first read・
zero-step・別module・changing input・cache解放、helper名shadow・source map・reachability、
Ref/cold/Failure/Defect/finalizerを確認します。`measure_display_runtime.py`の通常bodyを
使うtimingと、別runのcold/warm allocator計数を分けます。両最適化の最終統合では、
全体gateとcanonical compiler WASMの再生成も必要です。

| 変更範囲 | コマンド | 含まれる検証 |
| --- | --- | --- |
| sample metadata / source | `bun run check:sample` | native CLI sample実行、全sampleのWASM compile / format、Playground sample manifestのfreshness |
| sample / Playground UI | `bun run check:playground` | sample基盤、Playground / catalog lint、Tour manifest、Playground test、TypeScript typecheck、Vite build |
| Web UI browser review | `cd apps/playground && bun run test:visual` | Chromiumで全HTML sampleのviewport / interaction / image fallback / Code / Explorerを確認し、review PNGとHTML reportを出力 |
| Rust / compiler / LSP workspace | `bun run check:rust` | Rust format、workspace test（対象crateだけなら `bun run check:rust -- -p <crate>`） |
| production browser dogfood | `bun run check:production:browser` | canonical CLIでWeb starter / Flowをdevelopment/release buildし、Chromium操作・text/input値の一致、source-map defaults、JS例外を検証。Playground依存とChromiumの事前installが必要。full/release source gateにも含む |
| production artifacts | `bun run check:production` | first-party 6 fixtureのrelease build、manifest shape、必要runtime、size budget、digest / identity / 再build、process実行。full / release source gateにも含む |
| conformance fixture | `bun run check:conformance` | canonical conformance runner（対象rootを引数で限定可能） |
| Portable .d.ts converter | `bun run check:dts:wasm` | filesystem featureを無効にしたconverterをwasm32-unknown-unknownへbuildし、host importなしで変換・metadata・再生成report・診断を実行。WASI SDK 25.0とRust WASM targetが必要 |
| compiler/runtime/WASM boundary | `bun run check:wasm` | committed Playground WASMの再生成と差分確認。portable declaration parser用のWASI SDK 25.0が必要 |
| pure program WASM / typed callback | `cargo test -p seseragi-cli --test program_wasm` | 実program WASMをBun／Nodeで実行し、TS両profile／native対照、generated型宣言のstrict、標準package module/asset buildと失敗時の保持、生成virtual packageのowner、path依存・初期化、callback／instance寿命とtrap後のretireを検証。`rustup target add wasm32-unknown-unknown`が必要。workspace full gateでも省略しない |
| providerless Effect program WASM | `cargo test -p seseragi-backend-rust --test program_wasm_effect --test program_wasm_effect_admission --test program_wasm_private_ref` と上記CLI lane | checked型・source stage・式のadmissionを確認し、実WASMのcold実行・内部data Ref・型付き失敗・明示Defect・取消・masked finalizer・非同期dispose・返却callbackをfresh Bun／Nodeで実行する。pure-v1と別ABIで、host providerやtrap後unwindの保証を加えない |
| typed host Effect program WASM | `cargo test -p seseragi-backend-rust --test program_wasm_host_effect --test program_wasm_host_ref` と `cargo test -p seseragi-cli --test program_wasm_host_effect` | closed unary host callback、内部data Refのcapture、通常TS／nativeとのbody比較、実WASMのFailure／Defect・遅い応答後の取消・masked LIFO release・disposeとhost lease解放、標準packageの元のtyped bridgeを確認。一般provider ABIやtrap後unwindは含めない |
| pure program WASM package watch | `cargo test -p seseragi-cli --test dev` と `bun test ./runtime/ts/tests/program-lifecycle.test.ts` | 計算moduleの外側path依存とUIの再build、失敗時の完成site／version保持、元のdev lifecycle／authority検査を確認。実WASMの保存callback拒否・旧root0・遅いloadの回収をfresh Bun／Nodeでも実行。既存のTSのみのdev期限を延ばさない |
| VS Code extension | `bun run check:extension` | official ID / legacy migration boundary、extension lint・test、host向け正式VSIXのarchive mode・展開・`--version-json` smoke、非LSP migration VSIXのpackage / verify |
| Local Web product E2E | native archiveとVSIXをpackage後に`bun run test:local-web-e2e` | 展開したCLIとinstall済みVSIXだけを使い、canonical multi-module Web projectのLSP / Dev / browser / edit / diagnostic / recovery / production buildを同一sourceで確認し、reportとPNGを`target/local-web-product-e2e`へ保存 |
| release metadata / native artifact | `bun run check:release` | canonical version source、Cargo/JS/WASM version同期、CHANGELOG、host向けCLI / LSP archive・checksum・再展開実行smoke |
| tag release source gate | `bun run check:release-gate` | repository-wide full gate相当。ただしnative / VSIXのpackage smokeは同一SHAを使うrelease matrix jobへ委譲し、成果物を二重生成しない |
| repository-wide | `bun run check` または `bun run check:full` | format、lint、Rust workspace、全conformance、native samples、WASM、Playground、extension |

相互再帰するclosed EffectのTS注釈は、`cargo test -p seseragi-driver --test mutual_effect_returns`
と`cargo test -p seseragi-cli --test mutual_effect_returns`で確認します。元の生成TSを両profileでstrict検査し、
cold再実行・typed Failure・Defect後のscope解放をBun／Nodeとnativeで比較します。
共有Core解析を変える場合はlowering／driver crateの検証と、統合前の全体gateも必要です。

ConsoleのString契約はdriver／backendの`--test console_string_contract`とCLIの同名test、
`bun test runtime/ts/tests/console-string-contract.test.ts`で確認します。prelude／import／
保存したcallableの引数型、canonical ownerとscoped binder、raw生成TS両profileのstrict、
fresh Bun／Nodeとnativeの出力、cold再実行、引数評価時のDefectとscope解放を保ちます。
共有型互換性を変える場合はsemantics／driverの既存検証と最終統合gateも必要です。

HKTのADT payload・struct field・public aliasはdriverとCLIの
`--test hkt_data_projection`で確認します。元Issueの単独source、opaque aliasのfactory実値、
facadeと保存関数、元の生成TS両profileのstrict、fresh Bun／Nodeとnativeを比較します。
constructor binderを宣言する通常functionは同じchecked witnessを引数と返り値へ渡し、
制約付きfunctionの辞書段階と、別scopeの同名value binderを維持します。
外側のtype parameter名を内側で再使用する、仕様で禁止された宣言は位置付きで拒否します。
private fieldと偽造、別ownerやpayload違いの拒否を保ち、不正kindの位置付き診断は
driverの`--test hkt_annotation_kinds`でも確認します。一般のUnknown targetやHKT逆推論は
このlaneの受入へ追加しません。

標準primitive EqのTS投影は、`cargo test -p seseragi-cli --test string_equality`
と`--test inherited_equality`で確認します。異なるliteral・保存したmethod・部分適用を
元の生成TS両profileでstrict検査し、fresh Bun／Nodeで実行します。Int／String／Charの
native対照と、直接method・部分適用のsource一回評価・順序も保ちます。
Bool／UnitのEqと前置`!`のnative契約は、backend／CLIの`--test basic_scalars`と
上記の元literal fixtureで確認します。Unitも二つのsource引数を評価し、保存した部分適用、
別nominal ownerとscoped binderの拒否、Ref・cold再実行・Failure／Defect後の解放を保ちます。
前置operatorは関数値として参照できません。標準Float Eqは提供しません。
共有lowering／driverの検証に加え、統合前は全体gateとcanonical compiler WASMの検証が必要です。

native Clock sleepの有限契約は、backend／CLIの`--test clock_sleep`とbackend runtimeの
`clock_tests`で確認します。明示native CLI provider選択、canonical DurationとClockの
owner／kind／source引数、target receipt、通常path packageのTS用lock保持、元のTS両profileの
raw strict、fresh Bun／Nodeとnativeの結果を比較します。pending sibling、cold再実行、
typed Failure／Defect／取消、acquisitionのatomic登録とsleepを使うmasked finalizer、旧outputと入力保持、
timer／managed root解放を検証します。Clock now、WASMのClock row、一般provider ABIと
target別manifest／native provider lockはこのsliceに含めません。共通native schedulerを
変えるため、既存filesystem／Fiberとprogram WASMの回帰も確認し、統合時は全体gateを行います。

importしたADTのpayload identityと注釈付きEq部分適用は、driverの
`--test frontend_payload_identity --test frontend_inference --test imported_returns`で確認します。
constructorを公開するopaque ADT、local alias、同canonicalのdirect／facade／namespace import、
別ownerと不正な残り関数型の拒否を確認し、表示名でownerを推測しません。
CLIの`--test string_equality`と`--test nominal_lambda_identity`では、元の生成TS両profileの
strict、fresh Bun／Node、native、source引数の一回評価、cold再実行、Failure／Defect後の
finalizerとroot解放を比較します。FloatConversionErrorを含むnative fixtureには、
同errorのcanonical変換ABIが必要です。無注釈の演算子部分適用全般はこの修復に含めません。

generic implの演算子は、driverとCLIの`--test generic_impl_operators`で確認します。
生成instanceが元implのscoped binderを引き継ぎ、明示instance、別owner、importとreexport、
制約、overlapとfunctional dependencyの位置付き拒否を保つことを検証します。
元のTS両profileのstrict、fresh Bun／Nodeとnativeの結果、保存した演算子のsource一回評価、
Ref cycleとcold Effectの再実行、Failure／Defect後のfinalizerとroot解放も比較します。
一般aliasのhead mappingや、返却関数を続けて適用する未対応全般は含めません。

`do`末尾の分岐を含むcold Effectは、driverとCLIの
`--test deferred_effect_environment`で確認します。先に実行するstatementの環境・失敗型、
最終実行が返すEffect値の型、未実行のcold値を区別し、環境・成功型・失敗型の不正な
注釈を位置付きで拒否します。元のTS両profileのstrict、fresh Bun／Nodeとnativeで
再実行、先行処理の一回実行、Failure／Defect後のfinalizerとRef root解放を比較します。
共有frontendの修復なので、統合時は全体gateとcanonical compiler WASMの検証も必要です。

固定済みalias引数のconstructor sectionは、driverとCLIの
`--test alias_constructor_sections`で確認します。canonical ownerと宣言kindを根拠に、
残るslotが順序どおり一度だけ現れる場合に限って、並替・重複・消失済みprefixを投影します。
opaque providerのfacade、別owner・不正kind・未証明headの位置付き拒否、元のTS両profileの
strict、fresh Bun／Node、native、選択済みevidenceとsource一回評価、cold Effectと
Failure／Defect後の解放を比較します。未束縛の一般type lambdaやHKT逆推論は含めません。
共有frontendの変更なので、統合時は全体gateとcanonical compiler WASMの検証も必要です。

transparent aliasの残るvalue-kind parameterを並替・重複・破棄するconstructorは、
semanticsの`type_functions::tests`、loweringの`core::types::tests`、driverとbackendの
`--test alias_type_functions`、driver／CLIの`--test alias_constructor_sections`で確認します。
`Flip<A,B> = Pair<B,A>`、`Repeat<A> = Pair<A,A>`、`Erase<A> = Int`の明示type引数を使い、
scoped slotのcaptureを避けた適用、canonical owner／provider、alpha同値の重複診断、
renamed facade packageと保存methodのsource arityを保ちます。元のTS両profileのstrict、
fresh Bun／Nodeとnativeの出力・source一回評価、Ref・cold再実行・Failure／Defect後の
finalizerとroot解放を比較します。使わない束縛parameterもconstructorのarityに数え、
失われるfree instance parameter、不正owner／kind、未証明の選択は位置付きで拒否します。
このproducerはcanonical nominal application、scoped value slotと、各要素がvalue kindを
持つfunction・closed record・tupleを扱います。構造型aliasはdriverの
`--test structural_aliases --test structural_alias_facades`とCLIの
`--test alias_constructor_sections`で、renamed／namespace facade、返却method、元のTSの
strictと公開型consumerの負例、callbackのRef保存・cold再実行・Failure／Defect後の
finalizerとroot解放を確認します。共有semantics／lowering／driver全体の回帰も確認します。
CLIの`--test rust_backend alias_head`で元のfunction／record拒否入力がsourceを変えずに
emit／buildできることと、不正method型・失われるinstance parameterの位置付き拒否を確認します。
非対応という旧期待値を残さず、非実行・出力なし・source保持の既存assertionを保ちます。
row／requirement merge constructor、一般HKT逆推論、完全なtype-function同値判定と
program WASMの受入拡張は含めません。共有型表現・投影の変更なので、統合前は必要な
全体gateとcanonical compiler WASMの再生成・公開consumer検証も必要です。

制約付き関数やprimary辞書のmethodが返す関数の連続適用は、backendの
`--test returned_evidence_stages`とCLIの`--test alias_constructor_sections`で確認します。
宣言のsource境界・完全な返却型・owner／kind・辞書を検証し、各本体を次のsource引数より先に
実行します。保存・未使用部分適用、同名Unit／Int binder、Show aliasとUnit callback、
最初の段階のDefect、構造型alias経由のRef・cold再実行・Failure／Defect後の解放を比較します。
不正Core入力は位置付きで拒否し、program WASMの受入範囲は広げません。

各laneは `scripts/check-scoped.sh` を共通runnerとして使います。Playgroundのtest、
typecheck、buildはcatalog / Tour manifest確認をlane内で一度だけ済ませてから直接実行し、
`package.json`の個別scriptが同じcatalog確認を三回繰り返す構造を避けています。

`check:sample`はCLI実行対象だけでなくbrowser-interactive sampleもcommitted WASMで
project compileし、全sourceがformatterのcanonical outputと一致することを確認します。
manifest/hashだけが更新され、壊れたinteractive sourceが通過する状態にはしません。

sample・Tourのコマンド検証は、processの終了とstdout・stderrのEOFを両方確認します。
期限切れは独立した失敗として返し、後始末で再びstdioを無期限に待ちません。
POSIXでは親が先に終了しても所有process groupを停止します。親終了後の出力、
非zero終了、Unicode、期限と後始末は`scripts/sample-cli-process.test.ts`で確認します。

配布archiveの検証は、展開したCLI/LSPとarchive commandのprocess終了・stdioのcloseを
確認してから一時directoryを削除します。一時的な`EBUSY`等で失敗した場合も、削除を試みるのは
最初の試行を含めて最大6回です。再試行の待機は合計1500msまでです。
検証とcleanupが両方失敗した場合は両方のエラーを保持し、
検証が成功してもcleanupが失敗すれば失敗です。`bun test scripts/native-release.test.ts`で
正常・非zero・検証拒否・cleanup失敗と子process終了後の実ファイル削除を確認します。
cleanup失敗時は、実行したcommandのPID・終了code・exit/closeの観測も残します。
Windows固有のhandle競合の確認は、Windows CIと正式Releaseの実archive検証も必要です。

PostgreSQL Provider bundleの鮮度確認は、実際に解決された`node_modules`のsection comment
だけを正規化します。`bun test scripts/postgres-provider-bundle.test.ts`で通常installと
深いworktreeのsymlink installの生成bytes一致、実行コード変更時の拒否を確認します。

Playgroundのruntime resolverは`runtime/ts/package.json`のexportsと
`runtime/providers/package.json`の`runtime-browser/*` exportsから生成します。
`bun run --cwd apps/playground runtime:generate`で更新し、`runtime:check`と
Playground testsが生成物・compiler Runtime ABIのmodule/export・browser provider
manifestとのparityを検証します。host adapterはruntime packageの`browser` condition、
native専用moduleは`browser: null`で宣言し、Playgroundへ別のmodule一覧を足しません。
providerの`browser/` sourceから`runtime-browser/` packageへのprojectionはRustの
package staging testでも同一内容を検証します。

## Standard instance audit

Prelude auditの`specified-but-implementation-missing`行は、canonical registryの
`tracking_issue`に正のIssue番号を必ず持ちます。surface生成時に検証するため、
unit testだけでなくconformanceのartifact照合も未追跡行を拒否します。
GitHubへの問い合わせは不要です。実装完了時はmissing行を削除し、implemented /
structural / intentionally unavailable行に古いtracking metadataを残しません。

`prelude::surface`のsynthetic missing testは、現在のmissing一覧が空でもこの契約と
JSONの`trackingIssue`投影を検証します。auditだけの変更はこのsemantics testと
conformanceの`stdlib_surface` testを先に実行できます。

## Dependency installation

検証用のBunはCIとlocalで **1.4.4** に揃える。versionの基準はrootの
`.bun-version`で、GitHub Actionsも同じfileを読む。localでは`bun --version`と
`cat .bun-version`が一致することを確認してから検証する。

以前の1.3.9ではLinux x64 WASMのIPIntクラッシュと同じstackを全体gateで観測し、
[WebKit更新 #26922](https://github.com/oven-sh/bun/pull/26922)を含む1.3.11へ
揃えていた。1.4.4でBrotli decoderのCVE-2026-111497が修正されたため更新し、
この境界の実行も省略しない。
WASMの実行・assertionを省略したり、JSCの実行tierを無効化したりはしない。
既存Docsの過去の実測version表記は、その検証時点の記録として残す。

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

## Browser executable for Playground Bun tests

PlaygroundのBun browser testsは、既定でlockfileのPlaywrightに対応するChromiumを使います。
Cloudなどでインストール済みGoogle Chromeを使う場合は、その実行ファイルの絶対pathを
`SESERAGI_TEST_BROWSER_PATH`へ設定します。起動したbrowserのpathとversionをログへ残し、
指定pathで起動できない場合はそのまま失敗します。この設定はBun tests用で、
`test:visual`やproduction browser scriptsのbrowser選択は変更しません。

```sh
cd apps/playground
SESERAGI_TEST_BROWSER_PATH=/path/to/google/chrome bun test tests
```

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

full-page baselineはPlayground chromeとPreview / Editor / Explorerの位置関係を確認するため
維持します。ただし許容差分は画像全体の比率ではなく、各baselineが監視するPreview body、
Editor panel、Workspace shellの面積の1%をpixel数へ変換して判定します。色差threshold 0.25は
platform別font rasterizationを吸収し、局所surfaceのspacing / typography / alignment差分が
大きな余白へ薄まらないようにします。感度testは三種類の意図的な局所差分がこのbudgetを
超えて比較失敗になることを固定します。

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

### Release profile implementation checks

`cargo test -p seseragi-cli --test profiles` compiles the canonical performance
fixtures in both profiles and executes each in a fresh process. It also runs
`scripts/release-shapes.ts` against the emitted release module and fixture
predicates. Install root dependencies with `bun install --frozen-lockfile` first.
The checker resolves canonical module symbols through generated inspection roots;
metadata does not report optimization success. Its AST proof is conservative:
unsupported wrapper/projection shapes fail instead of being guessed. Inspection
roots are retained in release output; bundling/DCE is a separate boundary.

`bun test scripts/release-shapes.test.ts runtime/ts/tests/effect-stack.test.ts`
checks shape rejection cases and 100,000 Effect binds independently of pure TCO.
The conformance lane includes these checks. Profile/compiler/runtime integration
still requires the full gate and regenerated WASM during Promotion.

WASM再生成時は[WASI SDK 25.0](https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-25)のホストOS/CPU用archiveを展開し、`WASI_SDK_PATH`をそのdirectoryへ設定する。C parserのcompilerも固定し、Rust/wasm-packだけが同じでも発生する生成物差分を防ぐ。

WASM glueには`cargo install wasm-bindgen-cli --version 0.2.126 --locked`でCargo.lockと同じCLIを用意する。`wasm-pack`の自動install fallbackはCLI内部の依存versionを変えるため、buildは`--mode no-install`で既存のCLIを使う。

配布用WASMの正規生成hostはLinux x64。`check:wasm`は生成物全体のbyte一致を検証し、PRとrelease promotionのLinux gateで必須とする。Cargoはhostで実行するbuild script/proc macroのmetadataを依存先へ伝播するため、macOS生成物とはcode/dataを含む差分が生じ得る。macOS CIでは同じ固定toolchainと2つの独立したfresh Cargo targetで再構築する。1回目のfresh outputと2回目のrepo/pkg stagingで全packageのfile集合とbyte一致を確認し、変換・workspace・browser回帰を検証する。さらに両hostの変換JSON出力をCIで完全一致比較し、macOS生成物で配布artifactを置き換えない。

captureしたprimitiveの推論identityは、`cargo test -p seseragi-semantics --lib`、
`cargo test -p seseragi-driver --lib --test compile_module --test captured_primitive_identity`、
`cargo test -p seseragi-cli --test captured_primitive_identity`で確認する。
同名nominalの拒否とscoped evidenceを保ち、元のsourceとcold Effectの正常・Failure・Defectを
raw TS両profileのstrict、fresh Bun/Node、nativeとfinalizer・managed rootで比較する。
標準moduleを使うfixtureはstandaloneのsemantics helperへ渡さず、共有driver経由で検証する。
共有推論の変更には、統合前のfull gateと正規WASMの更新も必要となる。

nativeのString Eqは、`cargo test --locked -p seseragi-backend-rust --test string_equality --test equality_partials`と
`cargo test --locked -p seseragi-cli --test string_equality`で確認する。canonicalなEqのowner・型・evidenceを
変えた負例を拒否し、Unicode/NUL・保存method・部分適用・cold再実行・Failure/Defect・finalizer/root0を
raw strict TS両profile、fresh Bun/Nodeとnativeで比較する。file条件分岐では入力と失敗時の既存出力も保つ。
共有runtimeを使うため、通常Web buildのprogram WASMと元のtyped bridge、callback/instanceの解放も検証する。
左右のsource helperだけを計測するtraceで、一回評価と順序を比較し、生成Eq式は変更しない。
共有runtimeの統合にはfull gateと正規compiler WASMの更新も必要となる。

nativeのtext.lines/joinは、`cargo test --locked -p seseragi-backend-rust --lib text_tests`、
`cargo test --locked -p seseragi-backend-rust --test text_processing`と
`cargo test --locked -p seseragi-cli --test text_processing`で確認する。Unicode改行・CRLF・空入力と
残る文字列のbytes、保存joinの一回評価・順序、cold再実行・Failure/Defect・finalizer/root0を
raw strict TS両profile・fresh Bun/Node・nativeで比較する。fileの入力と読込失敗時の出力を保護し、
通常Web buildのprogram WASM、元typed bridgeのstrict・skipLibCheck:false、callback/instance解放も
確認する。公開Array codecや一般provider ABIはこの受入に含めない。共有runtimeの統合には
full gateと正規compiler WASMの更新が必要となる。

nativeの十進Int parseは、`cargo test --locked -p seseragi-backend-rust --lib int_parse_tests`、
`cargo test --locked -p seseragi-backend-rust --test int_parse`と
`cargo test --locked -p seseragi-cli --test int_parse`で確認する。標準のIntParseErrorのowner・kind・
source arity・evidenceを検証し、fileとpath packageの集計、入力と失敗時の既存出力の保持、
保存parseの一回評価、cold再実行・Failure/Defect・finalizer/root0をraw strict TS両profile・
fresh Bun/Node・nativeで比較する。標準IntParseErrorのShowはdriver/backendの
`--test int_parse_error_show`とCLIの`--test int_parse standard_parse_error_show`で確認する。
4つのconstructor、同名local型とpath package、元のraw strict TS両profile、cold失敗と
Defect/finalizer、fileの既存出力保持を比較する。radix指定、他の数値変換API、一般program WASM、
IntParseErrorのEq/Debugはこの受入に含めない。共有runtimeの統合にはfull gateと
正規compiler WASMの更新が必要となる。

nativeのFloat parseは、`cargo test --locked -p seseragi-backend-rust --lib parse_tests`、
`cargo test --locked -p seseragi-backend-rust --test float_parse --test int_parse --test int_parse_error_show`と
`cargo test --locked -p seseragi-cli --test float_parse --test int_parse`で確認する。
共通number parse ABIのowner・kind・source arity・evidence検査と既存Int/Showの拒否条件を保持し、
decimal/exponent構文、UTF-8 byte offset、構文失敗とoverflowの順序、subnormal・underflow・
signed zero・明示NaN/Infinityを確認する。sourceとpath packageで小数の各行を集計し、
元のraw strict TS両profile・fresh Bun/Node・nativeで比較する。入力と失敗時の既存出力、
保存parseの一回評価、cold再実行・Failure/Defect・finalizer/root0を検証する。
FloatParseErrorの新しいShow/Debug/Eq、他の数値変換API、一般program WASMはこの受入に含めない。
共有backend/runtimeの統合にはfull gateと正規compiler WASMの更新が必要となる。

nativeのFloat丸め・Int変換は、backendの`--lib float_conversion_tests`と
`--test float_conversion`、CLIの`--test float_conversion`で確認する。
canonicalなRoundingModeとFloatConversionErrorのowner/kind・source arity・evidenceを保ち、
6modeの中間値と隣接値、signed zero、safe Int境界と非finite失敗を比較する。
source/path packageの元生成TS両profileをstrict検査し、fresh Bun/Node/nativeで
入力と失敗時の旧出力、一回評価と保存・返り値converter、cold Failure/Defect・finalizer/root0を確認する。
program WASMは位置付き拒否を保ち、Float Eq/Ord・Pow/stdmathや新しいerror Showを含めない。
共有numeric ABIの変更には元Int/Float parseとShowの回帰確認、統合前の全体gateと正規compiler WASMも必要となる。

nativeのInt剰余は、`cargo test --locked -p seseragi-backend-rust --test int_remainder --test int_division --test operator_partials`と
`cargo test --locked -p seseragi-cli --test int_remainder`で確認する。標準Remのowner・kind・source arity・
選択済みevidenceを検証し、既存Divとprogram WASMの拒否条件を保持する。正負の値とsafe integer境界、
直接呼出し・保存method・generic/返り値関数の一回評価と順序、cold再実行、zero divisorのDefectと
finalizer/root0を、元のraw strict TS両profile・fresh Bun/Node・nativeで比較する。
sourceと凍結したpath packageで各行の整数の剰余を別fileへ書き、入力と失敗時の既存出力を保持する。
Pow、新しいchecked remainder API、一般program WASMへの剰余の追加は受入に含めない。
共有backend/runtimeの統合にはfull gateと正規compiler WASMの更新が必要となる。

nativeのInt/Float累乗は、backendの`--lib power`と`--test native_power`、CLIの
`--test native_power`で確認する。標準Powと`std/float.power`のcanonical owner/kind、
辞書、source arity、保存methodと返り値関数、同名のlexical binderを区別する。
Intのsafe integer境界・負指数とoverflowのDefect、FloatのNaN/Infinity/signed zeroと
指数の偶奇を、元生成TS両profileのstrict検査とfresh Bun/Node/nativeで比較する。
直接・保存した演算子のsource一回評価と順序、Refとcold再実行、型付きFailure/Defect後の
finalizer/root0、sourceと凍結したpath packageでの入力・失敗時の旧出力保持も検証する。
一般program WASMへの累乗の追加、残るstdmath API、Float Eq/Ordは含めない。
既存Div/Rem・Float parse/conversion・operator partialの回帰を確認し、共有backend/runtimeの
統合前は全体gateと正規compiler WASMの検証も必要となる。

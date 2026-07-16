# Seseragi 現在地

この文書は、新しいSeseragi仕様が「何を決め、どこまで検証され、何がまだ動かないか」を
一か所で確認するための非規範な進捗表です。言語の意味は [`spec/`](./spec/README.md) を、
機能ごとの検証対象は [`../examples/spec/COVERAGE.md`](../examples/spec/COVERAGE.md) を正とします。

## 現在の結論

新しいSeseragiは、言語と標準ライブラリの**仕様初稿がおおむね揃い、Rust再実装を縦sliceで進めている段階**です。
compiler全体は未完成ですが、lexer / SurfaceAst / resolver / typed HIR / CoreIr / TypeScriptIr / emitterと
versioned runtimeの最小経路は実際に動作しています。

lessonやfixture全体が実装済みという意味ではありません。実装済み範囲はRust conformance artifactを正規生成・比較し、
未接続のlesson / project fixtureは引き続き「新実装が満たすべき契約」です。

ロードマップは[`ROADMAP.md`](./ROADMAP.md)の「言語能力Phase × product surface」二軸で管理します。
CLI / LSP / playgroundを言語完成後の一括product化Phaseへ延期せず、同じdriverを検証する縦sliceとして進めます。

## 状態の意味

| 状態                | 意味                                                  |
| ------------------- | ----------------------------------------------------- |
| 仕様化済み          | `docs/spec/` に構文、型、評価、失敗、境界の規則がある |
| design exampleあり  | 学習用programまたは最小fixtureがある                  |
| fixture契約あり     | 入力と期待結果の形式が機械検証可能な形で置かれている  |
| conformance検証済み | 新仕様compilerを実際に通し、期待結果と比較できる      |
| 実装済み            | compiler、runtimeまたはtoolingが新仕様どおりに動く    |

`bun run check:spec-examples` が保証するのは、lessonの番号・前提・日本語説明・期待stdoutと、
fixture sidecarの形式・仕様節参照・diagnostic spanの整合です。`bun run conformance:artifacts` は
artifact bundleの発見、必須file、JSON envelope、参照snapshotの整合だけを検査します。Seseragi sourceの
parse、型検査、lowering、生成物比較はRust側の`cargo run -p seseragi-conformance -- .`が担当し、
Effectおよびpure execution fixtureについては生成moduleとversioned runtimeを実行します。

## 領域別の現在地

| 領域                                       | 仕様          | example / fixture                          | 新仕様実装         |
| ------------------------------------------ | ------------- | ------------------------------------------ | ------------------ |
| 基本文法、演算子、pattern                  | 初稿あり      | lessonあり、fixtureは一部                  | tuple / matchまで部分実装 |
| 型、generic、ADT、struct、record           | 初稿あり      | lessonあり、fixtureは一部                  | ADT / standard sum / rank-1 generic fnまで部分実装 |
| trait、Functor、Applicative、Monad、Monoid | 初稿あり      | Functor / Applicative / Monad実行fixtureあり、law fixture不足 | HKT推論 / local dictionary / transitive supertrait実行まで部分実装 |
| custom infix operator                      | 初稿あり      | compile fixtureあり                        | 未着手             |
| Effect、resource、concurrency              | 初稿あり      | lesson、時間制御・cleanup fixtureあり      | Console / Stdin + imported non-generic Effect call / positive project executionまで部分実装 |
| Signal、Stream                             | 初稿あり      | lessonあり、runtime fixture不足            | 未着手             |
| module、package、project                   | 初稿あり      | module graph・lock・manifest fixtureあり   | strict core manifest + canonical local discovery + linked compile / executionまで部分実装 |
| TypeScript interop、`.d.ts`変換            | 初稿あり      | load・ABI・変換snapshot fixtureあり        | 未着手             |
| collection、text、number、JSON             | 初稿あり      | Range / Array executionあり、他は境界fixture不足 | Array / `Range<Int>` reduceまで部分実装 |
| Bytes、Decimal、Regex、timezone            | 初稿あり      | lessonあり、fixture不足                    | 未着手             |
| filesystem、process、HTTP                  | 初稿あり      | cleanup・shutdown・body stream fixtureあり | 未着手             |
| pure HTML、SSR、DOM、hydration             | 初稿あり      | SSR lesson、DOM / hydration fixtureあり    | 未着手             |
| 性能モデル、最適化境界                     | 初稿あり      | shape・profile差分・stack fixtureあり      | 未着手             |
| diagnostics、formatter、LSP                | 縦slice進行中 | diagnostic fixture + stdio / CLI integration | LSP-0とformatter-0を実装              |
| syntax highlight                           | token契約あり | spec preview拡張あり                       | 仮実装あり         |
| playground                                 | 共有契約あり  | lesson 01 + Phase 1 WASM execution          | Playground-1 mobile / deploy surface実装済み |

ここでいう「未着手」は刷新仕様に対する状態です。repositoryに存在する現行compilerやruntimeのcodeを
削除した、または何も実装されていない、という意味ではありません。現行実装は設計資料として残って
いますが、新仕様への適合をまだ主張しません。

## いま存在する成果物

- 規範仕様: `docs/spec/00-language.md` から `14-performance.md` とAppendix grammar
- 学習教材: `examples/spec/lessons/` の31 lesson
- versioned conformance入力: positive / diagnostic / runtime artifactとproject fixture
- coverage表: `docs/SPEC_COVERAGE.md` と `examples/spec/COVERAGE.md`
- grammar対応表: `examples/spec/grammar-coverage.json`
- token / stage / execution artifact契約: `examples/spec/artifacts/token-schema-1/`、
  `schema-1/`、`stage-schema-1/`、`execution-schema-1/`
- 横断監査記録: `docs/SPEC_AUDIT.md`
- 構造checker: `scripts/check-spec-examples.ts`
- artifact runner skeleton: `scripts/conformance-artifacts.ts`
- Rust compiler crates: syntax、source、project identity、diagnostic、semantics、lowering、single-module driver、runtime / conformance boundary
- TokenStreamからgenerated TypeScriptまでのschema-1 artifact producer / conformance比較
- cold Effect valueをversioned TypeScript runtimeで実行するConsole / Stdin execution fixture
- typed ADT、tuple、match、exhaustivenessを通す`rock-paper-scissors-domain` artifact
- standard `Maybe` / `Either`と、正常・不正入力を実行する`parse-hand-either` artifact
- pure `parseHand`をcold typed Effectへ変換し、success payloadを実行比較する`effect-parse-hand` artifact
- ADT / match / typed failure / Stdin / Consoleを統合し、正常・不正・EOFを実行比較する
  `rock-paper-scissors-cli` artifact
- strict manifestの`run.entry`からrelative importを辿り、分割じゃんけんの3 source moduleをcanonical filesystem pathと
  structural package / module identityへ対応付けるlocal package loader
- 同じCLIでConsole operation trace、Stdin / Console host failureのtyped変換、derived `Show`によるstderrと
  exit code 1まで比較するexecution artifact
- cross-module ADT / pure callを実行する`project-schema-1/rock-paper-scissors-domain-split`、namespace constructorと
  generic callを実行する`namespace-generic-call`、imported cold EffectとConsole traceを実行する`imported-effect-console`
- imported typed failureをmainのADTへmapし、dependencyのderived `Show` dictionary import、stderr、exit 1まで実行する
  `project-schema-1/imported-effect-failure`
- 物理source pathと論理module identityを分離し、Phase 1の累積programをTokenStreamからgenerated
  TypeScriptまで一つのcompile結果として返すpublic Rust driver
- fixture metadataなしでPhase 1 sourceをcompile / executeする`seseragi run path/to/app.ssrg`と、compiler outputから
  entry contractを検証して埋め込みTypeScript runtimeを実行するpublic Rust runtime boundary
- 表示確認用syntax highlight: `extensions/seseragi-spec-preview/`

数は進捗の目安にすぎません。lessonが存在しても、対応するpositive / negative / runtime fixtureが
揃うまでは、その機能をconformance検証済みとは扱いません。

## まだ終わっていないこと

### 1. 仕様の横断監査

章ごとの初稿はありますが、同じ概念を複数章から参照する箇所について、型signature、failure、
resource lifetime、grammar、exampleの最終突合が必要です。最初の型構文・性能境界passは完了し、結果を
`docs/SPEC_AUDIT.md`へ記録しています。新機能を増やすより先に残りの矛盾を減らします。

### 2. conformance fixtureの拡充

single-file / project fixtureは継続的に拡充中です。特に次が不足しています。

- operator precedence、型推論、kind、coherenceのpositive / negative case
- Signal transaction、Stream backpressureのruntime trace
- DOM reconciliation、event cleanup、hydration mismatchのproject fixture
- trait specialization、fusionのquality profile fixture
- formatter、LSP、semantic token、playgroundで共有するsnapshot

### 3. 新仕様compilerの実装

実装計画は`IMPLEMENTATION.md`を正本とし、lexer / lossless CST、parser、名前解決、型検査、
意味を単純化した内部表現、TypeScript出力、runtime adapterを分離して進めています。
TypeScriptは出力先であり、Seseragiの型やEffectの意味を決める正本にはしません。

### 4. productとしてのtooling

single-file CLIはshared driverとstructured diagnosticsへ接続済みです。LSP-0も`seseragi-lsp`のstdio JSON-RPCから
同じdriverを呼び、UTF-8 / UTF-16 / UTF-32へ変換したparse / resolve / type diagnosticsをopen documentへ返します。
Playground-0も`seseragi-wasm`から同じdriverとruntime entry contractを利用し、lesson sourceを手作業で複製せず
WASM compile、generated TypeScript、browser host runtimeへ接続済みです。Phase 1累積じゃんけんもdeterministic Stdinで
browser host実行しています。formatter-0はlossless token / CST、shared driver、native CLIを接続し、Phase 1
累積programのround-tripを固定しました。resolved fixity依存の整形とrange / stdin formatは独立した後続surface
gateとして進め、Phase 2完了後へ一括延期しません。
package directoryの`seseragi run .`はlocal filesystem discovery、shared project driver、multi-module runtimeへ接続済みです。
manifestのlanguage rangeもsource読込前に実装versionへ照合します。path dependencyはcanonical rootとtyped manifestから
structural package graphへ解決し、dependency name不一致、cycle、同名同versionの別source混入を拒否します。各dependencyの
bare importは宣言済みdependency keyの最長prefixとtarget export mapを使い、未宣言dependencyを`SES-K0103`、非公開subpathを
`SES-N0104`で拒否します。root entryからrelative / `self/` / path dependency importを辿るcanonical source discoveryも
`ModuleIdentity` graphへ接続済みです。driverはopaque package scopeでpublic/private境界を維持し、packageごとのoutput pathを
計画します。CLIとruntimeは`package-path-dependency-basic`の公開function callを実行して`42`を確認済みです。entryから到達しない
`.ssrg`も含むsource root auditがNFC / case collision、root escape、physical aliasをcompile前に拒否します。registry /
lockfile resolutionとfull collection fixtureは後続Phase 2 gateとして残します。Array literal自体は通常pipelineへ接続済みで、
generic higher-order parameter callとchecked Int arithmetic operator section `(+)`も通常pipelineへ接続済みです。
低優先順位適用`$`と左結合pipeline`|>`もSurfaceAstで通常applicationへ消去し、部分適用を右辺に置くchainを
全IR、generated TypeScript、actual executionへ接続済みです。行頭`|>`による継続行はbind、pure let、
do blockのresultで同じ構文規則を使い、indentを意味へ含めません。専用runtime dispatchは生成しません。
Playgroundは同じfixtureを`関数適用とパイプライン`として実行し、サンプル選択を`基本`、`アプリ`、
`型と抽象化`へ分類しました。類似したconstraint実装fixtureはconformanceには残しつつ、利用者向けcatalogでは
代表例へ絞り、内部機構の差だけでサンプル一覧が偏らないようにしています。
標準`Reducible<Array<A>, A>`を使うgeneric `reduce`は`schema-1/array-reduce`で選択済みevidenceを
TypedHir / CoreIrへ保持し、checked `(+)` callback、TypeScript runtime、actual executionまで接続済みです。
Playgroundでも`Arrayスコア集計`として同じdriver / browser runtimeを実行できます。
`1..10`と`1..=10`は算術より低く比較より高いbinding powerを持つ`Range<Int>` literalとして通常pipelineへ
接続しました。両端はIntに限定し、誤った端点は`range.endpoint-not-int`でsource range付きdiagnosticを返します。
backendはRangeをArrayへ偽装せず、`start / end / inclusive`を持つ有限値と`std/range::Reducible` evidenceを保持します。
`schema-1/range-reduce`と同名execution fixtureはexclusive 45 / inclusive 55、空・降順・Int64最大端をruntime ABIで
検証します。Array / Range comprehensionは同じ`Iterable<C, A>` evidenceをTypedHir / CoreIrへ保持し、guardを
純粋なpredicate、複数generatorをnested flat-mapとしてTypeScript runtimeへlowerします。
`schema-1/{range-comprehension,array-comprehension}`は単一generatorとguard、およびArray同士の複数generatorを固定し、
`execution-schema-1/range-comprehension`とPlaygroundの`Rangeと内包表記`はeven square total 220を実行します。
非Iterable sourceは`SES-T0201 instance.missing`で停止します。constructor / tupleなど反駁可能patternのfilter semanticsと、
user-defined Iterable dictionary dispatchは後続gateです。
Int算術binaryとoperator sectionは`Add<Int, Int, Int>`などの選択済みevidenceをTypedHir / CoreIrへ保持し、
backendはそのevidenceを確認してchecked runtime helperを選択します。
`String + String`も`Add<String, String, String>`のstandard instanceとして全IRとactual executionへ接続済みで、
数値への暗黙変換は行いません。operator resultをIntだけに固定しない標準evidence selectionを証明し、
Playgroundでも`Stringで招待状`として同じfixtureを実行できます。
Int、Bool、Stringの`==` / `!=`もstandard `Eq<A>`の選択結果をTypedHir / CoreIrへ保持し、Stringの
pure comparisonをactual executionへ接続済みです。derived / user-defined Eqの比較生成は後続gateです。
残る主要gateはstandard instanceを列挙するだけでなく、user-defined arithmetic instanceを通常dictionary dispatchへ
接続することです。user-defined instanceのmethodはSurfaceAstで
signature / body / spanを保持し、resolverのinstance / method scopeで名前解決されるところまで接続済みです。public traitの
method contractもSurfaceAst / ModuleInterfaceで構造化schemeとして保持します。instance headと各`where` constraintの
trait名は独立したtrait namespaceでlocal / import / prelude symbolへ解決し、後続の契約照合が文字列比較へ戻らない境界まで
接続済みです。local user-defined instanceはmethodの不足・余分・重複・body不足・signature不一致をresolved symbol identityと
trait type argument substitutionで検査し、alpha-renamed generic methodと部分適用型構築子も契約一致として扱います。
imported public traitもdependency interfaceのmethod schemeを使い、provider nominal型のcanonical identity、
trait type argument substitution、alpha-renamed generic method、部分適用型構築子、prelude constraintを同じ契約modelで検査します。
provider-localまたはproviderがimportした別traitをmethod constraintが参照する場合も、provider interfaceからcanonical trait identityを
consumerのresolved importへ運び、consumer側の同名traitへ綴りで取り違えません。
TypedHir / CoreIr / TypeScriptIr / generated moduleのinstance headは、単一`head`型ではなくordered `arguments`として保持します。
これにより`Show<A>`の現在のdictionary経路を維持しながら、`Add<L, R, O>`や`Iterable<C, A>`を第一引数へ潰さず後続へ運べます。
`typeIdentity`はderived `Show`などprimary nominal typeを必要とする専用runtime consumer向けのoptional metadataであり、
一般instance identityの代用にはしません。module interfaceはcanonical `traitIdentity`とordered `argumentIdentities`も保持し、
同名traitやmulti-parameter instanceを`identity`文字列のparseへ戻さずtransportします。
instance method bodyにはtop-level pure `fn`と同じreturn type / call / conditional / array / match diagnosticsを適用し、
契約signatureだけ正しい不正bodyがTypedHir以降へ流れる経路を閉じています。
`project-schema-1/imported-trait-instance-contract`がclosed multi-module compilerの全IR / generated TypeScript gateを固定します。
local concrete user-defined instanceはmethod parameter / bodyをTypedHir、CoreIr、TypeScriptIrへ保持し、
`schema-1/user-instance-dictionary`でcustom dictionary objectを生成します。custom dictionaryはderived `Show`のruntime type importを
要求せず、canonical identityは全head argumentから構成します。同fixtureの`render value`はargument型からlocal concrete instanceを
選び、selected evidenceをTypedHir / CoreIrへ保持してTypeScriptのdictionary method callまで生成します。
`schema-1/trait-method-candidates`は同名methodをargument型とlocal instance evidenceで選び分け、選べないcaseを
`SES-T0202`に固定します。`schema-1/generic-instance-dispatch`はconstraintなしgeneric headの型引数を選択済みevidenceへ残し、
TypeScriptのdictionary factory具体化とmethod callまで生成します。
`schema-1/constrained-instance-dispatch`はconstraint付きgeneric local instanceに必要なlocal evidenceを再帰選択し、
TypedHir / CoreIrのevidence tree、TypeScriptIrのfactory argument、生成TSのdictionary引数、actual executionまで接続します。
instance method bodyのtrait callもresolved constraint scopeから`parameter` evidenceを選び、factory closureのdictionaryを
実際に消費します。循環evidenceは`instance.missing`で停止します。trait method固有の`where`も
`schema-1/method-constraint-dispatch`でprimary dictionaryの後ろへordered evidenceとして選択し、method closureの
compiler-private parameterから消費してactual executionまで接続済みです。instance-levelとmethod-levelのparameter indexは
offsetで分離します。未接続なのはstandard evidenceのfactory引数化とgeneric / constraint付きimported factoryです。
public constrained pure functionはmodule interfaceのconstraintをprovider-local canonical trait identityとともに運び、
consumer側のlocal instanceをimported callへ渡せます。`project-schema-1/imported-constrained-function`はproviderの
`describe<T> where Ready<T>`とconsumerの`Ready<Badge>`を通常project pipelineへ通し、生成ESMのclosed TypeScript check、
Console trace、stdoutまで固定します。`project-schema-1/imported-instance-dispatch`はproviderがexportしたconcrete
`Ready<Badge>` dictionaryを同じcallへ選択し、TypeScript source importとactual dispatchまで固定します。
`project-schema-1/transitive-instance-dispatch`はsource上はfacadeだけをimportするmainへoriginal provider dictionaryを計画し、
三moduleのclosed TypeScript checkとactual dispatchまで固定します。generic imported instanceとconstraint evidenceを持つ
imported factoryは未接続です。
local generic pure functionの`where`はbody scopeと飽和callへ接続済みです。
`schema-1/constrained-function-dispatch`はbodyの`parameter` evidence、call siteのlocal dictionary選択、
生成TSの末尾implicit dictionary parameterを固定し、execution fixtureが実際のdispatch結果を観測します。
first-class partial constrained functionとgeneric / conditional imported dictionary selectionは未接続です。

`F<_>`は通常type parameterとは別にarityを持つtype-constructor parameterとしてSurfaceAstとModuleInterfaceへ
保持します。`schema-1/functor-maybe`は`F<A> ~ Maybe<Int>`から`F = Maybe`と要素型を推論し、generic
`transform<F<_>, A, B> where Functor<F>`のbodyではparameter evidence、call siteではlocal
`Functor<Maybe>` dictionaryを選択します。選択結果はTypedHir / CoreIr / TypeScriptIrへ残り、
`execution-schema-1/functor-maybe`が`Just 41 |> transform increment`の`Just 42`をactual executionで固定します。
`type-constructor-kind-mismatch`は`Type -> Type`を要求するtrait parameterへ`Int : Type`を渡すinstanceを
`trait.instance-kind-mismatch`で拒否します。TypeScriptはHKTを直接表せないため、型検査済みの`F<A>` parameter
annotationだけをbackend境界で`unknown`へ消去しますが、Seseragiのkind、constraint、dictionary選択は消去しません。
`schema-1/applicative-maybe`は`Applicative<F>`が要求する`Functor<F>`をinstance contractとして検査し、
親dictionaryをApplicative factoryへ渡して生成dictionaryへspreadします。したがって
`where Applicative<F>`だけを持つgeneric functionでも、同じparameter evidenceから`map`、`pure`、`apply`を呼べます。
`lifted increment`のような制約付きcallを別のHKT callの引数へ置いた場合も、外側の期待型と左から確定した引数型を
使って`F = Maybe`を具体化し、内側と外側の両方へdictionaryを渡します。
`execution-schema-1/applicative-maybe`は`mapped`、`lifted`、`applyWrapped`を組み合わせた`Just 42`を固定します。
`schema-1/monad-maybe`はsupertrait chainを`Monad<M> -> Applicative<M> -> Functor<M>`へ伸ばします。
Monad factoryは具体化済みApplicative dictionaryを継承し、`where Monad<M>`だけを持つgeneric functionから
`pure`と`flatMap`を同じparameter evidenceで呼び出せます。calleeとcallerの型構築子parameterが同じ綴りでも
別scopeに属する場合、実引数で確定したsubstitutionを明示的に保持するため、入れ子の`pure value |> flatMap f`も
未解決型へ戻りません。`execution-schema-1/monad-maybe`は成功するbind列と`Nothing`のshort-circuitを実行します。
同じfixtureで`f <$> value`、`wrapped <*> value`、`value >>= f`をそれぞれ`map f value`、
`apply wrapped value`、`flatMap f value`へSurface ASTでdesugarし、通常の名前解決・型検査・dictionary dispatchを
再利用します。3演算子は`|>`と同じ低優先順位・左結合で、先頭演算子による複数行継続もformatterまで固定済みです。
pure functionの`do`は宣言済みreturn typeから`M<_>`を求め、`Monad<M>` evidenceを一度選択して
TypedHir / CoreIr / TypeScriptIrへ保持します。bindは選択済みdictionaryの`flatMap`、pure letはcontinuation内の
通常のbindingへlowerするため、Effect専用`Sequence`やeffect runtime helperを再利用しません。
`addMaybe (Just 20) (Just 22)`と`addMaybe (Just 20) Nothing`のactual executionが成功と短絡を固定します。
`semantic-diagnostics-schema-1/monad-do-invalid`はrefutable bind pattern、異なるmonad constructor、
final monadic expression欠落を、それぞれ専用の`do.*` diagnosticとsource rangeで固定します。
Applicative / Monad law、部分適用型構築子を使うdoは次の独立gateです。

Playground-1は`apps/playground`へ旧UIと分離して実装しました。CodeMirror 6、専用Seseragi highlight、
mobile panel、任意Stdin、driver diagnosticsのsource range表示を持ち、Vercel buildはreview済みWASM artifactを
静的bundleするためRust installを要求しません。旧`playground/`はrollback比較用に残しますが、root commandと
deployment configは新surfaceを正とします。
2026-07-14にVercel Git buildの成功、`application/wasm` asset配信、本番UIからのlesson 01実行を
<https://seseragi.vercel.app/>で確認しました。
local custom traitのvertical sliceは`Traitバッジ`としてsample catalogにも追加し、同じWASM driverとbrowser runtimeで
dictionary dispatchの実行結果を確認できます。
constraintなしgeneric local instanceのvertical sliceも`Generic instance`として追加し、`Maybe<Int>`に対する
generic dictionary factoryの具体化と、`Nothing` / `Just`両方のdispatchを同じsurfaceで実行できます。
constraint付きgeneric local instanceも`Constraint付きinstance`として追加し、必要なlocal evidenceを持つ
`Maybe<Badge>`の`Nothing` / `Just`両方を同じWASM driverとbrowser runtimeで実行できます。
local constrained functionとtrait method固有constraintもsample catalogへ追加し、通常のvalue引数に続く
compiler-private dictionary parameterと、primary method dictionaryに続くordered evidenceを同じWASM実行で確認できます。

Formatter-0は`seseragi-formatter`へline layout責務を分離し、`seseragi-driver::format_module`をCLI / LSP /
playgroundが再利用できるpublic entrypointにしました。`seseragi format`はfile I/Oだけ、`--check`は差分判定だけを
所有します。syntax errorはcompilerと同じsource range付きdiagnosticsを返し、recovery nodeをformatter都合で
書き換えません。現段階ではtoken順とintra-line triviaを保持する保守的なlayout formatterであり、12.8の
fixity-aware operator spacing、100 columns wrapping、range / stdin optionは未完了です。

### 5. 一般機能の未定義項目

standard API監査で再発見した`BigInt`の公開surface未定義は、`std/big-int`のmodule API、arithmetic
semantics、TypeScript境界、cost contract、lesson、compile fixtureへ移して解決しました。現時点で再登録中の
未定義項目はありません。

## 次の進行順

Phase 1のsingle-file累積programは完了gateを満たしました。次は同じprogramを捨てず、module / packageの
一般機構を加える順で進めます。

1. project resolverがpackage rootとsource rootからcanonical module identityを決め、driverへ物理pathとは別に渡す。
2. project layerが`ModuleGraph`のtopological orderでdependency interfaceをcompileし、generated output pathから
   `TypeScriptOutputPlan`を構築する。driverのlinked compile APIと、projectのentry / dependency output pathからimporter相対
   specifierへ変換するhelper、閉じたgraphをcompileする`compile_project`、backend側のalias、同名export、type-only edge、source
   map contractは固定済み。graphは実cycle witnessを返し、driverはgraph/source edge不一致、extra input、global output path衝突を
   拒否する。`project-schema-1`のconformance/writerで分割RPS domainを全IR・生成artifactまで固定し、planned output pathへ
   stageしたmodule setのTypeScript type-checkも行う。pure entry、imported Effect / Console entryに加え、domain / input / mainへ
   分割したじゃんけんCLIの正常・不正・EOF・Stdin host failure・Console host failureを同じ生成module setからBunで実行済み。
   project descriptorの複数case discoveryは実装済み。core manifestのpackage / layout / exports / runに加え、short registry、
   alias付きregistry、local path dependencyをtyped modelへparseできる。canonical local module discoveryとmanifest entryから
   一package graphを構築するloaderに加え、path dependency manifestを再帰解決する複数package identity graphも接続済み。
   package importからexact dependency identity / export subpathへの解決、path dependencyをまたぐsource graph、shared driver compile、
   generated runtime executionと全source root identity auditまで接続済み。full collection fixtureの分解を開始し、
   immutable Array literal、generic higher-order callback、checked Int arithmetic operator sectionは全IRとgenerated codeへ
   接続済み。標準Arrayのgeneric `reduce`もregistry / lockfile resolutionとは独立にactual executionまで回収済み。
   Int算術とoperator sectionは`Add<Int, Int, Int>`等のevidenceを全IRへ保持済み。次は標準名専用の経路を
   一般instance機構の完了と誤認せず、user-defined / imported instance searchへ進む。
3. 分割じゃんけんCLIはsingle-file版と同じtyped failure、Effect、derived `Show`、全五execution caseの結果を保持済み。
4. direct dependencyとfacade越しのconcrete user-defined evidenceはcanonical trait / argument identitiesでResolvedAstから
   TypedHir / CoreIr / TypeScript source import / driver output plan、actual executionまで保持済み。次はgeneric substitutionと
   constraint materializationでimported dictionary factoryを完成させる。
5. imported public callableのschemeに現れるnominal typeは、direct / transitive provider、namespace選択、異なるownerの同名typeを
   canonical identityで区別し、必要なtype-only outputをprovider closureから計画済み。provider欠落をlocal typeへfallbackしない。
6. imported trait method contract、local concrete dictionary dispatch、同名trait methodのlocal candidate選択、
   unconstrained / constrained generic local dictionary factory、local constrained function、method固有constraintは接続済み。
   次は同じordered evidence表現をcross-module selectionへ拡張する。nested namespace、constraint付きhigher-order callable、generic imported ADTは、それぞれ
   一般機構を証明する独立gateで回収する。

namespace-qualified constructor expression / patternとimported ADT exhaustivenessは、小さいsemantics / lowering fixtureと
`project-schema-1/namespace-generic-call`の実行経路まで接続済みです。このため次の累積goalではnamespace機能を増やすこと自体を
目的化せず、Phase 1のCLIを複数moduleへ移したときに必要になるEffect contractとinstance closureを優先します。

## 完了と呼ぶ条件

言語機能を「完了」と呼べるのは、次をすべて満たしたときです。

- 規範仕様とAppendix grammarが一致する。
- 学習用exampleがあり、前提順に読める。
- 必要なpositive / negative / runtime fixtureがある。
- 新仕様compilerがfixtureを実際に検証する。
- formatter、LSP、highlight、playgroundが同じfrontendを使う。
- host resourceやinteropを含む場合、cleanupとfailureが検証される。

この条件を満たす前は、「仕様化済み」「fixtureあり」のように到達点を限定して表現します。

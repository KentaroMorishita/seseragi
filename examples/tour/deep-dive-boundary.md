# Tour本編とDeep Diveの境界

この文書は、canonical Tourで必修にする理解と、任意のDeep Diveへ送る理論・
設計背景の境界を定めます。現行Tourの正本は[`curriculum.json`](./curriculum.json)、
surface coverageは[`coverage-report.md`](./coverage-report.md)です。

## 決定

Deep DiveはTour後半のcategoryには追加せず、TourともDiscoverとも独立した任意の
学習surfaceとします。`Learn`というtop-level navigation、tab、catalogは追加しません。
Tour完了後の案内、関連lessonからの任意リンク、stable direct URLから入れますが、
Deep DiveをTourのprerequisite、required topic、progressへ含めません。

理由は次の通りです。

- TourはHello worldから小さなapplicationまでを一方向に進む必修経路である。
- Deep Diveは既に使えるsurfaceの意味、法則、解決規則、設計判断を掘り下げる任意経路である。
- 理論説明の追加でTourの到達時間や163 surfaceのcoverage契約を増やさない。
- Deep Dive同士には独自の順序とprerequisiteを持たせられる。
- Recipe、Showcase、Referenceの既存責務を概念解説の置き場へ変えない。

`Tutorial`という二つ目の名前は追加しません。このrepositoryではcanonical Tutorialを
`Tour`と呼びます。Deep Diveは新しい必修Tutorialではありません。

## 判定基準

次のいずれかに該当する内容はTour本編に残します。

1. 通常のSeseragi sourceを左から読み、型と実行結果を予測するために必要である。
2. 公開syntax、type、operator、基本APIを初めて使うために必要である。
3. 後続lessonのsourceで前提として使う。
4. named operationとoperatorなど、通常codeで両方現れる表記を相互変換する。
5. 誤りをcompiler diagnosticから直すために必要である。

次のいずれかだけに該当する内容はDeep Diveへ送ります。

1. surfaceを使った後で、その法則や設計理由を説明する。
2. compiler内部のevidence、dictionary、resolution、loweringを説明する。
3. lawfulな抽象や大規模architectureを自分で設計する判断を扱う。
4. 同じ公開surfaceのperformance、scheduling、lifetime実装を掘る。
5. 知らなくても通常codeの読解、編集、diagnostic修正ができる。

両方に見える場合は、最小の利用契約をTourに置き、理由と内部構造だけをDeep Diveへ
分離します。Deep Diveの都合で未実装surfaceをsourceへ導入しません。

## Generic・Trait・抽象

| Topic | Tour本編の到達点 | Deep Diveへ送る内容 |
| --- | --- | --- |
| Generic function | concreteな重複をtype parameter付き関数へ置き換える | parametricityと型から導ける性質 |
| Generic data | generic ADTを宣言・構築・matchする | varianceや表現選択の理論 |
| 通常のtype parameterと`F<_>` | `A`とtype constructor parameterの適用形を読める | kindの体系とhigher-kinded abstractionの設計 |
| Trait | 「使えるoperationの契約」としてmethod signatureを読む・書く | dictionary elaboration、supertrait設計、契約の最小性 |
| Instance | 自作型へinstanceを定義し、選択結果を利用する | coherence、orphan境界、local / imported evidenceの解決詳細 |
| `where` | 必要なTraitをgeneric functionの型へ明記する | constraint simplificationとevidence passing |
| `impl` / custom operator | nominal typeへmethodとoperatorを定義して使う | API設計、fixity選択、operatorを公開する判断 |
| Functor | concreteな`map`と`<$>`を同じoperationとして書き換える | identity / composition lawとlawful instance設計 |
| Applicative | intermediate `F<A -> B>`を追い、`apply`と`<*>`を書き換える | Functorとの関係、独立計算の法則、評価戦略 |
| Monad | 依存する計算を`flatMap`、`>>=`、`do`で書き換える | Applicativeとの関係、law、do desugaringの詳細 |
| container比較 | Array / List / Maybe / Either / Effectで結果のshapeを比較する | 同じ抽象を選ぶ設計判断とcustom abstractionの作り方 |

具体型ごとの初出とTour外instanceの判断は
[`standard-instance-coverage.md`](./standard-instance-coverage.md)を正本とします。
現行`abstraction` categoryの17 lessonはoperatorの初出ではなく、前半で使った具体的な
operationを共通Trait contractと型ごとのsemanticsへ回収する本編として残します。

## Signal

Tour本編には次を残します。

- `make`、`read` / `*`、`set` / `:=`、`update`とread-only境界
- `map` / `<$>`、`pure`、`apply` / `<*>`によるderived Signal
- transaction結果が途中状態を公開しないという利用契約
- `switchMap`による動的依存とhandlerへ更新権限を閉じるownership
- Signalには通常のMonad operationを提供しないという公開surface上の境界

Deep Diveではdependency graph、revisionとscheduling、glitch freedomの成立条件、動的依存の
切替、subscription lifetime、そして「なぜSignalへMonadを持たせないか」という設計理由を
扱います。これらはruntime実装を知らなくても通常のSignal codeを書けるため任意です。

## Web UI

Tour本編にはpure `Html<Action>`、typed props / style、function component、link / image、
typed event、form、Signal view、`dom.run`、Typed Action、accessibility、feature state ownershipを
残します。利用者は本編だけで一画面のtyped formを読み、変更し、diagnosticを直せます。

Deep DiveではDOM reconciliation、keyとidentity、event delegation、IME event ordering、
cleanup lifetime、SSR escapingの内部境界、large UIでのcomponent / Action分割判断を扱います。
個別tagやAPIのsignature一覧はDeep DiveではなくReferenceです。

## Application

Tour本編にはconsole reportのdata → transform → fallibility → Effect outputと、Web UIの
static view → component → Signal → form event → Action → validation → feature ownershipを
残します。各stepは単独で実行でき、通常applicationのpure / effect境界を追えます。

Deep Diveではmodule数が増えたときのarchitecture trade-off、feature間protocol、抽象を
Traitとして切り出す時期、evidence境界、test strategy、performance観測を扱います。
目的別に完成sourceを変形する内容はRecipe、統合済み完成形はShowcaseに置きます。

## 教材surfaceの責務

| Surface | 問い | 内容 |
| --- | --- | --- |
| Tour | 最初に何をどの順で学ぶか | 必修surface、実行、変更、diagnostic、到達課題 |
| Deep Dive | なぜそう設計され、どこまで一般化できるか | 法則、内部意味論、設計判断、architecture trade-off |
| Recipe | 既習概念を目的へどう適用するか | 一つの課題を解く実行可能な変形例 |
| Showcase | 複数surfaceを統合すると何が作れるか | 完成したapplicationと探索可能なstate |
| Reference | 正確なsignatureや制約は何か | compiler由来の検索可能な型・API情報 |

同じtopicが複数surfaceへ現れる場合も本文を複製しません。Tour / Deep Diveは前提となる
conceptへ、Recipe / Showcaseは利用したlessonへ、Referenceはcompiler-owned identityへ
linkします。

## Tour本編だけで保証する到達点

Deep Diveへ進まなくても、利用者は次を行えます。

- generic function / data、Trait、instance、`where`、`F<_>`を含む型を読む・書く
- `map` / `apply` / `flatMap`と`<$>` / `<*>` / `>>=` / `do`を相互に書き換える
- Maybe / Either / EffectのfailureとSignalの時間変化を型に保つ
- typed formとActionをpure reducer、Signal、DOM Effectへ接続する
- feature ownershipを保った小さなconsole / Web applicationを変更する
- compiler diagnosticのcode、range、messageから教材上の誤りを直す

この保証は`requiredTopics`、prerequisite / required surface検証、全source / exercise /
diagnostic gate、browser interaction testで維持します。Deep Dive追加時もこの完了条件を
Tour側から移しません。

## 後続実装の単位

Deep Dive導線を実装するときは、一つの追跡Issueで次を同時に扱わず、少なくとも
「独立導線とdata model」と「初期Generic / Trait教材」に分けます。Signal、Web UI、
ApplicationのDeep Diveは初期教材のformatとnavigationが確定してから別Issueにします。

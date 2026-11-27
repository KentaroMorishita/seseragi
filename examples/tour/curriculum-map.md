# Tour カリキュラム構成（本編 35 ステップ＋補足トラック 21 ステップ）

この文書は、`A Tour of Seseragi` の現行カリキュラム構成（全 11 カテゴリ・15 章・56 レッスン）を一覧にまとめたものです。データ定義の実体は [`curriculum.json`](./curriculum.json) にあり、Tour の画面・ルーティング・進捗管理・前後移動はすべて `curriculum.json` から生成されます。

## 設計方針

- **1 ステップ 1 概念**: リテラル種別のような細かすぎる単位は統合し、`Effect`・`Signal`・`match` の網羅性・`Trait` など理解の段差が大きい概念は独立したステップで丁寧に扱います。
- **本編と補足トラックの分離**: 入門から小さな Web アプリの完成までを一本道で学べる **本編コア（Steps 01〜35）** と、抽象の法則・演算子・型設計・Web UI やコレクションの細目を必要に応じて学べる **補足トラック（Steps 36〜56）** に分けています。
- **手を動かす演習**: 各レッスンはそのまま `Run` できるコード・行範囲と対応した解説・自分で書き換えて確かめる演習・よくある間違いとエラー表示例で構成します。
- **安定 ID**: カテゴリ・章・レッスンは表示順とは独立した安定 ID を持ち、前提レッスン（`prerequisites`）を明示します。

## カテゴリ一覧

### 本編コア（Steps 01〜35）

| 順 | カテゴリ ID | タイトル | 章数 | ステップ範囲 | 到達内容 |
|---:|---|---|---:|---|---|
| 1 | `foundations` | 基本の構文と値 | 1 | 01–03 | プログラムの実行、`let` と型推論、`if` 式による条件分岐 |
| 2 | `functions-and-pipelines` | 関数とパイプライン | 1 | 04–05 | `fn` による関数定義、部分適用、`|>` と `$` による処理の連結 |
| 3 | `data-and-collections` | データの形とコレクション | 2 | 06–12 | タプル・無名レコード・`struct`・ADT・`match` と網羅性、`Array`・`Range` と集計パイプライン |
| 4 | `fallibility` | 値の有無と失敗（Maybe と Either） | 2 | 13–16 | `Maybe` による値の有無の表現、`Either` による失敗理由の伝達と変換 |
| 5 | `effects` | 副作用と Effect | 1 | 17–21 | 純粋関数と `Effect` の分離、`do` 記法、`fails` と短絡、`mapError`/`recover`、`with` |
| 6 | `abstraction` | ジェネリクスと Trait | 1 | 22–23 | 型パラメーター `<A>` / `<A, B>`、`deriving Eq/Show` と `where` 制約 |
| 7 | `signals` | 時間変化する値（Signal） | 1 | 24–27 | `Signal` / `MutableSignal` の読み書き、派生 `Signal`、`transaction` による一括更新 |
| 8 | `web-ui` | Web UI | 1 | 28–32 | 静的 HTML・スタイル、コンポーネント関数、型付き Action、フォームイベント、`dom.run` |
| 9 | `applications` | まとめアプリと次のステップ | 1 | 33–35 | 検証付き集計コンソール、対話型 Web UI の総合演習、修了後の案内 |

### 補足トラック（Steps 36〜56）

| 順 | カテゴリ ID | タイトル | 章数 | ステップ範囲 | 到達内容 |
|---:|---|---|---:|---|---|
| 10 | `side-trips-abstraction` | 補足トラック：抽象の法則・演算子・型設計 | 2 | 36–47 | `Functor`・`Applicative`・`Monad`、`Signal` の境界と `switchMap`、`impl`・カスタム演算子、kind・一意性・法則・最小境界 |
| 11 | `side-trips-collections-web` | 補足トラック：Web UI とコレクションの細目 | 2 | 48–56 | 外部リンクと画像、アクセシビリティ属性、機能単位の状態管理、`List`・`reduce` 設計・`append`/`concat`・内包表記・空コレクション |

## レッスン一覧

### 本編コア（Steps 01〜35）

#### 1. 基本の構文と値（`foundations`）

- **章 1: 実行・値・分岐（`foundations-basics`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 1 | `01-hello-world` | 最初のプログラムを動かして書き換える | コードを書き換えて `Run` し、文字列テンプレートの結果を出力できる | なし |
| 2 | `02-values-and-bindings` | 値に名前を付ける（`let` と型推論） | `let` で値に名前を付け、型注釈の有無と基本演算の結果を説明できる | `01-hello-world` |
| 3 | `if-expression` | 条件で値を選ぶ（`if` 式） | `if` 式で条件に応じた値を返し、`else` の必須性と型一致を説明できる | `02-values-and-bindings` |

#### 2. 関数とパイプライン（`functions-and-pipelines`）

- **章 1: 関数定義とパイプライン（`functions-core`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 4 | `03-function-definitions` | 関数を作って呼び出す（`fn` と部分適用） | `fn` で関数を定義し、引数適用と部分適用で新しい関数を作れる | `if-expression` |
| 5 | `05-pipelines` | パイプラインと `$` で処理をつなぐ | `|>` と `$` で値の流れる順に複数の関数を組み合わせて出力できる | `03-function-definitions` |

#### 3. データの形とコレクション（`data-and-collections`）

- **章 1: データの形と `match`（`data-shapes-and-match`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 6 | `tuple-and-record` | タプルと無名レコード | タプルとレコードを作り、参照やスプレッド更新・幅部分型を使える | `05-pipelines` |
| 7 | `06-records-and-structs` | 名前付きデータ `struct` | `struct` を定義して無名レコードと使い分け、名義型で取り違えを防げる | `tuple-and-record` |
| 8 | `adt-definition` | 代数的データ型（ADT） | `type` で状態ごとの構築子を持つ代数的データ型を定義できる | `06-records-and-structs` |
| 9 | `07-adts-and-patterns` | `match` で値を分解する | `match` の分岐で ADT から中身の値を取り出し、変数や `_` で安全に処理できる | `adt-definition` |
| 10 | `exhaustive-match` | 網羅性チェック | `match` の漏れをエラー表示で確認し、全ケースを網羅して修正できる | `07-adts-and-patterns` |

- **章 2: `Array`・`Range`・集計パイプライン（`collections-basics`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 11 | `08-collections-and-ranges` | `Array` と `Range` の基本 | `Array` と `Range` を作り、境界を意識して安全に要素を取り出せる | `exhaustive-match` |
| 12 | `collection-pipeline` | `map`・`filter`・`reduce` で集計する | `Array` に対して変換・絞り込み・集約をパイプラインで組み立てられる | `08-collections-and-ranges` |

#### 4. 値の有無と失敗（`fallibility`）

- **章 1: `Maybe` で値の有無を表す（`fallibility-maybe`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 13 | `maybe-match` | `Maybe`（値の有無を表す） | `Maybe` の値を `match` で分岐し、`??` で既定値へ取り出せる | `collection-pipeline` |
| 14 | `maybe-short-circuit` | `Maybe` の `map`・`apply`・`flatMap` | `Maybe` の `map`・`apply`・`flatMap` と演算子表記を使い分け、`Nothing` の短絡を確認できる | `maybe-match` |

- **章 2: `Either` で失敗理由を伝える（`fallibility-either`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 15 | `09-maybe-and-either` | `Either`（失敗の理由を表す） | `Either` の値を `match` で分岐し、理由の有無に応じて `Maybe` と `Either` を使い分けられる | `maybe-short-circuit` |
| 16 | `either-map-error` | `Either` の成功と失敗を別々に変換する | `Either` の成功側と失敗側を分けて変換し、失敗情報を保ったまま合成できる | `09-maybe-and-either` |

#### 5. 副作用と `Effect`（`effects`）

- **章 1: `Effect` の組み立てと失敗の扱い（`effects-basics`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 17 | `effect-pure-vs-effect` | 純粋な関数と `Effect` | `fn` と `effect fn` の役割の違いを区別し、純粋な計算を `Effect` から呼び出せる | `either-map-error` |
| 18 | `10-effects-and-do` | `do` 記法で `Effect` を順につなぐ | `do` と `<-`、`succeed` を組み合わせて順番実行を書け、`flatMap` や `>>=` と対応付けられる | `effect-pure-vs-effect` |
| 19 | `effect-failure-short-circuit` | `Effect` の失敗（`fails`）と短絡 | `fails` で失敗の型を宣言し、失敗時の短絡動作を実行して確認できる | `10-effects-and-do` |
| 20 | `effect-map-error` | `Either` と `Effect` の変換・エラー変換 | `Either` の値と `Effect` の失敗をつなぎ、`mapError` と `recover` でアプリ側の型に揃えられる | `effect-failure-short-circuit` |
| 21 | `effect-with-capability` | 必要な機能（`with`）を宣言する | `with` に必要な環境機能を並べ、省略時との型の違いを読み取れる | `effect-map-error` |

#### 6. ジェネリクスと `Trait`（`abstraction`）

- **章 1: 型パラメーターと `Trait`（`abstraction-traits`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 22 | `abstraction-type-parameter` | ジェネリクス（型パラメーター `<A>` と `<A, B>`） | 型パラメーター `A` や複数の型パラメーターを持つ関数と汎用データ型を読み書きできる | `effect-with-capability` |
| 23 | `abstraction-trait-operation` | `Trait`（`deriving Eq/Show` と `where` 制約） | `deriving` で `Eq` と `Show` を使えるようにし、`where` 制約で必要な振る舞いを指定できる | `abstraction-type-parameter` |

#### 7. 時間変化する値（`signals`）

- **章 1: `Signal` の基本と更新（`signals-derivation`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 24 | `signals-value-difference` | `Signal` と `MutableSignal`（作る・読む `*`） | `MutableSignal` を作成し、`Signal` としての参照や `*` 演算子による読み取りを書ける | `abstraction-trait-operation` |
| 25 | `signals-write-operator` | `Signal` を書き換える（`:=`・`update`） | `MutableSignal` を `set`・`:=`・`update` で安全に書き換え、前後の変化を追える | `signals-value-difference` |
| 26 | `signals-combine` | 派生 `Signal`（`map`・`combine`） | 単一や複数の `Signal` から派生する `Signal` を組み立て、元値の更新が反映されることを確認できる | `signals-write-operator` |
| 27 | `signals-handler-boundary` | `transaction` でまとめて更新する | `signals.transaction` と `planSet` で同時更新を行い、更新権限と表示用 `Signal` を分けられる | `signals-combine` |

#### 8. Web UI（`web-ui`）

- **章 1: HTML・コンポーネント・対話の仕組み（`web-ui-pure-html`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 28 | `web-html-value` | 静的 HTML とスタイル・属性 | `html` 名前空間と `style`・子要素配列を使って純粋な HTML ツリーを組み立てられる | `signals-handler-boundary` |
| 29 | `web-component-props` | コンポーネント関数で部品化する | 引数付きで `Html` を返すコンポーネント関数を定義し、親要素の中に組み込める | `web-html-value` |
| 30 | `web-typed-action` | クリック操作と型付き Action | `Action` 型と `onClick`、`match` による状態更新関数を組み合わせて書ける | `web-component-props` |
| 31 | `web-form-submit` | 入力フォームとイベント | 各種フォームイベントからそれぞれの型付き Action を生成して接続できる | `web-typed-action` |
| 32 | `web-dom-run` | `Signal` を画面につなぐ（`dom.run`） | `Signal<Html<Action>>` を `dom.run` へ渡し、動く画面としてブラウザ上で動かせる | `web-form-submit` |

#### 9. まとめアプリと次のステップ（`applications`）

- **章 1: 総合演習と次の案内（`applications-capstone`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 33 | `applications-console-data` | 総合演習 1: 検証付き集計コンソール | 純粋な検証・整形処理と `main` の `Effect` 出力を分離したコンソールプログラムを書ける | `web-dom-run` |
| 34 | `14-integrated-app` | 総合演習 2: 型付き Action の対話型 Web UI | 純粋な状態更新関数と `Effect` 境界で状態を更新する対話型 Web UI を実行できる | `applications-console-data` |
| 35 | `design-learning-map` | 次のステップ（Discover・ドキュメント・補足トラック） | Tour の修了後に目的に応じて Discover・ドキュメント・補足トラックを選び進める | `14-integrated-app` |

### 補足トラック（Steps 36〜56）

#### 10. 補足トラック：抽象の法則・演算子・型設計（`side-trips-abstraction`）

- **章 1: 共通の枠組み・固有メソッド・演算子（`side-trips-abstraction-concepts`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 36 | `abstraction-functor-map` | 具体型の `map` と `<$>` を `Functor` で統一する | 複数の具体型で使った `map` と `<$>` を `Functor` の共通な型として説明できる | `abstraction-trait-operation`, `collection-pipeline`, `maybe-short-circuit`, `either-map-error` |
| 37 | `abstraction-applicative-apply` | `Maybe` の `apply` と `<*>` を `Applicative` で統一する | `Maybe` で使った `apply` と `<*>` を `Applicative` の共通な型として説明できる | `abstraction-functor-map`, `maybe-short-circuit` |
| 38 | `abstraction-monad-bind` | `Maybe` と `Effect` の `bind` を `Monad` で統一する | `Maybe` と `Effect` で使った連鎖の表記を `Monad` の共通な型として説明できる | `abstraction-applicative-apply`, `maybe-short-circuit`, `10-effects-and-do` |
| 39 | `abstraction-type-comparison` | 同じ操作を複数の型で比べる | 同じ操作が異なる文脈の形を保ったまま働くことを確認できる | `abstraction-monad-bind` |
| 40 | `abstraction-signal-contract` | `Signal` が `Monad` ではない理由を理解する | `Signal` で `<$>` と `<*>` は使えて `>>=` は使えない理由を説明できる | `abstraction-type-comparison`, `signals-handler-boundary` |
| 41 | `signals-switch-map` | `switchMap` で依存先の切り替えを明示する | `switchMap` と `>>=` の違いをコードと動作から説明できる | `abstraction-signal-contract` |
| 42 | `abstraction-impl-method` | `impl` で型に固有のメソッドを追加する | ジェネリック型へ固有のメソッドを定義して呼び出せる | `signals-switch-map` |
| 43 | `abstraction-custom-operator` | `infix` でカスタム演算子を定義する | 独自の演算子を定義し、中置記法と関数値の両方で使える | `abstraction-impl-method` |

- **章 2: 型設計の背景（`side-trips-abstraction-design`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 44 | `design-type-constructor-kinds` | `A` と `F<_>` の型引数の数（kind）を読む | `Type` と `Type -> Type` を区別し、型パラメーターによる制約が保つ形を説明できる | `design-learning-map`, `abstraction-custom-operator` |
| 45 | `design-trait-evidence` | `trait` の実装選択と一意性を追う | 汎用関数でどの `instance` が選ばれるかと、実装の一意性が保つ性質を説明できる | `design-type-constructor-kinds` |
| 46 | `design-abstraction-laws` | `Functor`・`Applicative`・`Monad` の法則を確かめる | 必要な抽象の強さを選び、法則が支える置き換え可能性を実行して確認できる | `design-trait-evidence` |
| 47 | `design-trait-boundary` | 最小の `trait` 境界を切り出す | 似た名前ではなく置き換え可能な最小操作から `trait` を設計できる | `design-abstraction-laws` |

#### 11. 補足トラック：Web UI とコレクションの細目（`side-trips-collections-web`）

- **章 1: Web UI の細目（`side-trips-web-ui`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 48 | `web-link-image` | 外部リンクと画像を表示する | `html.parseWebUrl` を使ってリンクと画像を安全に表示できる | `web-component-props` |
| 49 | `web-accessibility-label` | ラベルとアクセシビリティ属性を付ける | 入力欄と `label` を対応付け、状態の通知属性を設定できる | `web-link-image`, `web-form-submit` |
| 50 | `web-feature-state` | 部品の分割と状態の管理を保つ | 状態管理の境界を保った対話フォームと表示コンポーネントを組み合わせられる | `web-accessibility-label`, `web-dom-run` |

- **章 2: `List` とコレクションの細目（`side-trips-collections`）**

| 順 | レッスン ID | タイトル | 到達目標 | 前提 |
|---:|---|---|---|---|
| 51 | `list-values` | `List` を組み立てる | `List` をリテラルと cons 演算子で組み立てられる | `collection-pipeline` |
| 52 | `list-decomposition` | `List` を先頭と残りに分解する | `List` を `match` で先頭要素と残りの `List` に分解できる | `list-values` |
| 53 | `collection-reduce-step` | 集約の計算関数を書く | `reduce` に渡す集約用関数の引数の順と戻り値を設計できる | `list-decomposition` |
| 54 | `collection-append-concat` | `append` と `concat` でコレクションをつなぐ | `append` と `concat` の違いを区別してコレクションを連結できる | `collection-reduce-step` |
| 55 | `collection-comprehension` | 内包表記で一覧を作る | 内包表記の要素生成・絞り込み・出力式を読み書きできる | `collection-append-concat` |
| 56 | `empty-collections` | 空のコレクションを扱う | 空の配列に型注釈を付け、安全な既定値と組み合わせて扱える | `collection-comprehension` |

## Tour と他教材・検証との対応

- **網羅性の自動検証**: `curriculum.json` の `requiredTopics` と各レッスンの `introducedSurfaces` / `requiredSurfaces` / `prerequisites` の整合は `bun run check:tour` で自動検証し、対応表を [`coverage-report.md`](./coverage-report.md) へ出力します。
- **Deep Dive・Discover との役割分担**:
  - 必修の Tour と任意の理論・設計解説の境界は [`deep-dive-boundary.md`](./deep-dive-boundary.md) を参照してください。
  - 標準 `Functor` / `Applicative` / `Monad` インスタンスの配置は [`standard-instance-coverage.md`](./standard-instance-coverage.md) に記録しています。
  - 目的別の実用レシピや完成アプリ例は Playground の Discover（`examples/samples/`）で扱います。

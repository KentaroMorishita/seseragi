# コンパイラーの構成

コンパイラーの開発に、はじめて参加する人に向けた文書です。
ソースが実行できるプログラムになるまでの段階と、それぞれを受け持つクレートをまとめます。

- 言語の意味は[仕様](./spec/README.md)に従います。この文書は実装の説明で、規則ではありません。
- 適合を確かめる入力は `examples/spec/` にあります。
- Rust を出力する仕組みは [`architecture/rust-backend.md`](./architecture/rust-backend.md)、API ドキュメントの出力は [`architecture/api-document.md`](./architecture/api-document.md) にあります。
- これからの予定は [`ROADMAP.md`](./ROADMAP.md) にあります。

## 方針

- コンパイラー、フォーマッター、言語サーバー（LSP）、色分け、Playground が、同じ構文解析と型検査を使う。
- Seseragi の型、Effect、資源、モジュールの意味を、TypeScript の型検査に任せない。
- TypeScript は出力先の1つで、Seseragi の意味を決める層ではない。
- 各段階は、決まった形の中間の成果物と fixture でつなぐ。後の段階が未完成でも、前の段階を確かめられる。

## 段階

```text
SourceSnapshot
  -> TokenStream
  -> LosslessCst
  -> SurfaceAst
  -> ModuleInterface + ResolvedAst
  -> TypedHir
  -> CoreIr
  -> TypeScriptIr または Rust のコード
  -> 出力（TypeScript、ソースマップ、モジュールの情報）
```

| 成果物 | 持つもの | 持たないもの |
| --- | --- | --- |
| `SourceSnapshot` | 正規化した URI、UTF-8 のバイト列、版 | トークン、行と列の計算の結果 |
| `TokenStream` | 空白とコメントを含むトークンの種類、UTF-8 の範囲、書いたとおりの綴り | 演算子の優先順位、名前の解決 |
| `LosslessCst` | エラーの節も含め、すべてのトークンを持つ木。子の順は変わらない | 型、省略記法の展開、宣言の同一性 |
| `SurfaceAst` | 付録の文法に沿った宣言、式、パターンと、ソースの範囲 | import 先の宣言、推論した型 |
| `ModuleInterface` | 公開する名前、型、演算子の優先順位、インスタンスの頭、非推奨の情報 | 関数の本体、出力先での表現 |
| `ResolvedAst` | 宣言の一意な ID、スコープ、import のつながり | 型推論の結果、実行時の配置 |
| `TypedHir` | 型、種、制約を満たす根拠、型の変換、すべての場合を扱っているか | TypeScript の構文、最適化の形 |
| `AnalysisDocument` | 診断、宣言とスコープ、式の型、呼び出せる値、標準ライブラリの説明 | 出力、実行、エディターの画面 |
| `CoreIr` | 評価の順を明示した小さな意味の核。Effect の操作、クロージャー、代数的データ型 | 省略記法、ホストのモジュールの綴り |
| `TypeScriptIr` | JavaScript と TypeScript の ABI、実行時のライブラリの呼び出し、非同期の境、ソースマップの元の位置 | Seseragi の型推論の判断 |

- 段階の変換は、前の段階の値を壊さず、新しい値を返します。
- 各節は、1回の実行の中で変わらない ID と、UTF-8 のソースの範囲を持ちます。
- 後の段階はソースを読み直しません。書いたとおりの綴りが必要なら、前の段階の成果物から受け取ります。
- `SurfaceAst` から直接 TypeScript を出す近道は作りません。必ず `CoreIr` を通します。

成果物の形は fixture で確かめます（`examples/spec/artifacts/`）。確かめ方は [SCOPED_CHECKS.md](./SCOPED_CHECKS.md) の「コンパイラーの成果物」にあります。
成果物を書き出す形式はテストと調査のためのもので、言語の ABI ではありません。同じ形式の主版を読む側だけが読みます。内部の型の名前やメモリーの番地は出しません。

## クレート

```text
crates/
  seseragi-source          ソース、行の位置の表、UTF-8・UTF-16・UTF-32 の位置の変換
  seseragi-syntax          字句解析、CST、SurfaceAst、構文の診断、モジュールの公開情報の形
  seseragi-semantics       名前の解決、型と種の推論、型クラス、網羅性、AnalysisDocument
  seseragi-lowering        CoreIr への変換、TypeScript の出力、ソースマップ
  seseragi-formatter       CST からの整形
  seseragi-project         パッケージとモジュールの同一性、seseragi.toml、ロックファイル
  seseragi-provider        実行環境に依存しない Provider Contract の形式と検証
  seseragi-interop         ソースの言語によらない外部の型の意味
  seseragi-projection      共通の意味から作る、アプリ向けの変換
  seseragi-schema          JSON Schema の読み取り
  seseragi-dts             .d.ts の変換（native とブラウザーで共有）
  seseragi-driver          段階を順に呼ぶ、純粋なコンパイラーの入口
  seseragi-runtime         コンパイルしたプログラムを実行する境
  seseragi-backend-rust    CoreIr から Rust のコードを出力する
  seseragi-native-runtime  native のプログラムが使う値と操作
  seseragi-release         公式ツールの配布の情報
  seseragi-cli             コマンドライン
  seseragi-lsp             言語サーバー
  seseragi-wasm            Playground 向けの WASM の入口
  seseragi-conformance     成果物と fixture を確かめる実行器
runtime/
  ts/                      生成した TypeScript が使う実行時のライブラリ
  providers/               Bun、Node、ブラウザー、PostgreSQL、SQLite などの provider
```

### 依存の向き

- `seseragi-source` から `seseragi-lowering` までは、ファイル、プロセス、Node、ブラウザーの API に依存しない純粋な処理です。
- `seseragi-driver` は、すでに特定したモジュールを、段階の順に処理します。ファイルの探索、パッケージの同一性、import のつなぎは `seseragi-project` の役目です。物理的なパスから論理的な同一性を決めません。
- `seseragi-cli`、`seseragi-lsp`、`seseragi-wasm` は同じ `seseragi-driver` を呼ぶ薄い入口です。構文解析、名前の解決、型検査、診断の作成を複製しません。
- `seseragi-runtime` は、構文解析、名前の解決、型検査、変換をしません。`seseragi-driver` の出力を受け取り、`main` の約束を確かめ、版つきの TypeScript の実行時のライブラリを用意して、実行環境のアダプターを呼びます。
- `runtime/ts` は Rust のクレートに依存しません。版つきの ABI の fixture だけを共有します。
- `runtime/ts` は生成したコードが今使う実行時のライブラリで、コンパイラーの代わりではありません。

Rust を選んだのは、ソースの言語を Rust に似せるためではありません。構文解析、型検査、中間表現のデータの持ち主をはっきりさせ、native の CLI と LSP、Playground の WASM で同じ処理を共有しやすいためです。

## 入口ごとの役目

### CLI

- `seseragi-cli` は、ファイルの入出力とコマンドの解釈だけを受け持ちます。
- コンパイルは `seseragi-driver` を、診断の表示は共通の端末向けの描画を、実行は `seseragi-runtime` を呼びます。
- `seseragi-runtime` は、コンパイルの結果から `main` の Effect の型、Console と Stdin の要求、選んだ `Show<E>` を確かめます。fixture の `run.json` は読みません。

### 言語サーバー

- `seseragi-lsp` は、標準入出力でのメッセージの区切り、LSP の要求の形、機能の取り決め、診断の変換だけを受け持ちます。
- 開いた文書の版ごとに、`analyze_module` で解析の結果を1度だけ作ります。ホバー、補完、シグネチャの表示、定義へ移動、診断の修正案、意味に基づく色分けは、この結果から答えます。
- 名前空間の補完は、解決した import と標準モジュールの公開情報を使います。ローカルの宣言と標準ライブラリの説明は、Playground と同じ同一性、型、説明を返します。
- コンパイラーの中の範囲は UTF-8 のバイトのままです。`seseragi-source::LineIndex` が、取り決めた UTF-8・UTF-16・UTF-32 の位置へ正確に変換します。行の外、文字の途中、サロゲートの途中を指す要求は、サーバーのエラーにせず、空の応答を返します。
- VS Code の拡張機能（`extensions/seseragi`）は、コンパイラーを複製しない薄い `vscode-languageclient` です。環境ごとの VSIX に同梱した native の LSP を起動します。
  - 起動の前に版を照合し、初期化の結果、ステータスバー、出力のパネル、再起動のコマンドで、実行ファイルの見つけ方、互換性、落ちたときの回復を見えるようにします。
  - 古い拡張機能の ID は移行用の拡張機能だけが持ち、LSP を起動しません。

### 診断

- `seseragi-syntax::Diagnostic` が、表示の唯一の元です。メッセージ、ラベル、注記、ヘルプ、修正案、期待した型と実際の型を JSON にできます。
- 端末はソースの抜き出しと `^` の印に、LSP は関連情報とデータに、Playground は位置へ移動できるカードに変えるだけです。
- 内部の `messageKey` は、回帰の fixture と分類のために残します。表示の文字列としては使いません。

### フォーマッター

- 共有のトークンと CST だけを入力にします。規則は [FORMATTER_LAYOUT.md](./FORMATTER_LAYOUT.md) にあります。
- `seseragi-driver::format_module` が、構文の診断とフォーマッターをまとめます。CLI はファイルの入出力と、書き込むか確かめるだけかを受け持ちます。
- 整形の前後でコンパイルし、`TypedHir`、`CoreIr`、`TypeScriptIr`、生成した TypeScript が変わらないことをテストで確かめます。

### Playground

- `seseragi-wasm` は、Playground が使う JSON の境だけを受け持ちます。構文解析、名前の解決、型検査、変換、Effect の入口の約束の解釈はしません。
- `seseragi-wasm::analyze_single_file` は純粋な解析だけを公開します。Playground の遅延した解析、ホバー、診断、検索できる説明は、同じ解析の結果を使います。JSON の形は `analysis-schema-1/shared-queries` の fixture で確かめます。
- 生成した TypeScript を、ブラウザー用の Console と Stdin の provider で実行します。コンパイラーや Effect の意味を作り直しません。
- 診断の範囲は、共有の UTF-8 のバイトの範囲のまま持ち、画面との境でだけ UTF-16 に変えます。
- 実行できるサンプルは `examples/samples/<slug>/` に、情報、ソース、説明、期待する出力をまとめます。同じソースを WASM と native の CLI の両方で実行して確かめます。フォルダーを1つ加えるだけで一覧に載ります。

**画面の構成**

- Playground（自由に書いて実行する）、Tour（順に学ぶ唯一の道すじ）、Discover（目的から Recipe や Showcase を探す）の3つに分けます。
- Playground は最初に `hello-world` を開きます。New blank で空の `main.ssrg` を作れます。Reset は今の元のソースに戻します。手元に保存した内容は、ひな形より優先して復元します。
- Discover は、題、要約、話題で検索でき、種類、話題、使う機能、注目と新着で絞り込めます。

**実行の隔離**

- Playground と Tour のプログラム、実行時のライブラリ、Effect と DOM は、`allow-same-origin` のない、別の出どころとして扱われる iframe の中で実行します。
- 親のページとは専用の MessageChannel で、結果、取消、宣言したアプリの Storage の操作だけをやりとりします。Storage には既存の名前空間と上限を当てはめ、Navigation は Preview の中の独立した履歴を使います。
- フォームのイベントを作るために `allow-forms` を許しますが、文書の CSP の `form-action 'none'` で外への送信と既定のページ移動を拒否します。
- 出どころを分けても、CPU の実行時間の上限は保証しません。

**フォームのイベント**

- `std/web/html` の公開情報が唯一の約束です。`InputEvent` と `ChangeEvent` は、中身を隠した読み取り専用の値として、解析、型検査、生成した TypeScript を通ります。
- ブラウザーの実行時のライブラリは、ネイティブの DOM のイベントを公開しません。対象の `value` と `checked` を1度だけ読んだ、変更できない値を変換の関数に渡します。
- `click`、`input`、`change`、`submit` は、取り付けた根に1度だけイベントの受け取りを登録し、描き直すときはイベントの対応表を入れ替えます。`submit` は、Msg を列に入れる前に `preventDefault` を同期的に呼びます。

**WASM をコミットする理由**

- Vercel のビルドで Rust、rustup、wasm-pack をインストールすると、環境の違いやダウンロードの失敗で配備が壊れます。
- そのため `apps/playground/src/wasm/pkg` は、例外として版つきの配備用の成果物としてコミットします。Vercel は、固定したロックファイルと Vite だけでビルドします。
- コンパイラーや WASM のアダプターを変えたときは `bun run build:playground:wasm` で作り直し、同じコミットに含めます。手で直してはいけません。
- `bun run test:playground:wasm` は、作り直した後の Git の差分も確かめ、古いままの成果物を CI で見つけます。

## 決まった形の成果物を変えるとき

次の形は、変えると多くの段階と fixture に影響します。内部の項目を足すのはかまいませんが、意味を変えるときは、読む側と fixture を同じコミットで直します。

1. トークンの種類、空白とコメント、UTF-8 の範囲、字句のエラー
2. CST の節の種類、足りないトークン、エラーの節、元のソースへの戻し方
3. 診断コード、主な範囲と関連する範囲、修正の編集
4. 正規化したモジュールのパス、宣言の ID、演算子の優先順位を含む、モジュールの公開情報
5. 型、種、制約の表示と、変数の名前の付け直しによる正規化
6. CoreIr の評価の順、失敗、取消、ソースの元の位置の決まり
7. TypeScript の実行時のライブラリの ABI の版と、生成したモジュールの情報

## 開発の進め方

- 1つの成果物の形には、受け持つ人を決めます。
- 成果物の形を変えるときは、形と fixture、それを読む側、作る側の順に小さく取り込みます。作る側の都合が、形の約束に漏れないようにするためです。
- 読む側に必要な項目は、作る側の中に直接足さず、先に問い合わせか成果物の項目のテストを加えます。
- 共有の列挙型に機能ごとの値を足す前に、既存の拡張の仕組みで表せない理由を書きます。
- fixture の期待値を変えるだけで、意味の変更を通しません。根拠となる仕様か不具合の説明を同じ変更に含めます。
- 新しい機能は、1人が文法の fixture から CST・AST、名前と型、CoreIr、出力、フォーマッターと LSP まで縦に通します。段階ごとに人へ受け渡しません。
- 特定のサンプルのための分岐をコンパイラーや実行時のライブラリに入れません。サンプルの名前を特別扱いせず、一般の機能の組み合わせで動かします。

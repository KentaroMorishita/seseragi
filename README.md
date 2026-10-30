<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./assets/brand/extension/logo-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="./assets/brand/extension/logo-light.svg">
    <img src="./assets/brand/extension/logo-light.svg" alt="Seseragi" width="720">
  </picture>
</p>

<p align="center"><strong>データの形から状態、UIまでを、ひとつの言語として組み立てる。</strong></p>

<p align="center">
  <a href="https://seseragi.vercel.app/">Playground</a>
  · <a href="https://seseragi.vercel.app/tour/">Tour</a>
  · <a href="./docs/README.md">言語仕様</a>
  · <a href="./extensions/seseragi/README.md">VS Code</a>
</p>

Seseragi（せせらぎ）は、データの形と処理の流れが、コードを読むだけで分かることを目指したプログラミング言語です。
型でデータを定義し、パターンマッチで場合分けし、パイプラインで処理をつなぎます。
副作用や失敗も型に現れるので、関数のシグネチャを見ればその関数が何をするのかが分かります。

<!-- canonical-example: path=examples/samples/fizzbuzz/main.ssrg -->
```seseragi
fn fizzBuzz number: Int -> String =
  // A tuple lets one match classify both divisibility rules.
  match (number % 3, number % 5) {
    (0, 0) -> "FizzBuzz"
    (0, _) -> "Fizz"
    (_, 0) -> "Buzz"
    _ -> `${number}`
  }

pub effect fn main = for number <- 1 ..= 30 {
  println $ fizzBuzz number
}
```
<!-- /canonical-example -->

```text
1
2
Fizz
4
Buzz
...
```

> [!NOTE]
> Seseragi は開発中の実験的な言語です。構文や仕様は今後も変わる可能性があります。

## 特徴

### データの形を型で書き、漏れはコンパイラが見つける

取りうる値を `type` で列挙し、`match` で場合分けします。
扱っていないケースがあれば、実行する前にコンパイラが教えてくれます。

```seseragi
type Shape =
  | Circle Float
  | Rect (Float, Float)

fn area shape: Shape -> Float = match shape {
  Circle radius -> 3.14 * radius * radius
  Rect (width, height) -> width * height
}
```

`Rect` の行を消すと:

```text
error[SES-T0301]: This match does not cover every possible value
  | fn area shape: Shape -> Float = match shape {
  |                                 ^ missing patterns: Rect _
  = help: Add the missing pattern arms, or add a final _ arm when a catch-all is intended.
```

### 処理は左から右へ流れる

`|>` で値を次の関数へ渡していきます。
関数はカリー化されているので、引数の一部だけを渡した関数をそのままパイプラインに置けます。

```seseragi
pub effect fn main =
  [Circle 1.0, Rect (2.0, 3.0)]
    |> map area
    |> sum
    |> show
    |> println
// => 9.14
```

### 副作用は型で区別される

画面への出力やファイルの読み書きのような副作用は、`effect fn` の中でしか起こせません。
ふつうの `fn` で `println` を呼ぶと、型エラーになります。

```seseragi
fn greet name: String -> Unit = println `Hello, ${name}`
```

```text
error[SES-T0101]: Expression type does not match the expected type
  = expected: Unit
  = actual: Effect<{ console: Console }, ConsoleError, Unit>
```

エラーの中の `Effect<{ console: Console }, ConsoleError, Unit>` は、
「コンソールを使い、`ConsoleError` で失敗することがあり、成功すると `Unit` を返す処理」という意味です。
必要なものと失敗の種類が、型を見るだけで分かります。

### 失敗も型の一部

失敗の種類は自分で `type` として設計し、`fails` で宣言します。
例外はありません。失敗を処理しないまま放っておくと、それも型に残ります。

```seseragi
import * as effects from "std/effect"

type OrderError =
  | OutOfStock String
  | TooMany Int

effect fn reserve item: String -> quantity: Int -> String fails OrderError =
  match item {
    "tea" -> fail $ OutOfStock item
    _ -> if quantity > 10 then fail $ TooMany quantity else succeed `${quantity} x ${item}`
  }

fn explain error: OrderError -> String = match error {
  OutOfStock item -> `${item} is out of stock`
  TooMany quantity -> `cannot reserve ${quantity} at once`
}

// 失敗をすべて処理したので、この関数は失敗しない（fails Never）
effect fn order item: String -> quantity: Int -> String fails Never =
  reserve item quantity
    |> effects.recover (\error -> succeed $ explain error)
```

副作用のない計算では、失敗するかもしれない値を `Maybe` や `Either` で表します。
`do` 記法を使うと、失敗しうる処理を上から順に書けます。途中で失敗したら、そこで止まります。

```seseragi
import * as either from "std/either"
import * as int from "std/int"

fn parseNumber text: String -> Either<String, Int> =
  int.parse text |> either.mapLeft (\_ -> `not a number: ${text}`)

fn divide total: Int -> count: Int -> Either<String, Int> = match count {
  0 -> Left "cannot divide by zero"
  _ -> Right (total / count)
}

fn calculate left: String -> right: String -> Either<String, Int> = do {
  total <- parseNumber left
  count <- parseNumber right
  divide total count
}

// calculate "84" "2"    => Right 42
// calculate "84" "zero" => Left not a number: zero
// calculate "84" "0"    => Left cannot divide by zero
```

### UI も同じ言語で

画面も、ここまでと同じ型と関数で書きます。
状態は `Signal` に持たせ、`signals.map` で状態から画面を作ります。
ボタンを押して状態が変わると、画面も自動で更新されます。

```seseragi
import * as dom from "std/web/dom"
import * as html from "std/web/html"
import * as signals from "std/signal"
import { MutableSignal } from "std/signal"

fn label current: Int -> html.Html<Task<Unit>> = html.p { children: `Count: ${current}` }

fn view count: MutableSignal<Int> -> current: Int -> html.Html<Task<Unit>> =
  html.div {
    children: [
      label current,
      html.button {
        onClick: signals.update (\n: Int -> n + 1) count,
        children: "+1"
      },
      html.button { onClick: signals.set 0 count, children: "Reset" }
    ]
  }

fn perform action: Task<Unit> -> Task<Unit> = action

pub effect fn main -> Unit with dom: dom.Dom fails String = do {
  count <- signals.make 0
  target <- dom.query "#app"
    |> mapError (\error: dom.DomError -> "#app not found")
  dom.run (dom.defaultOptions ()) target perform (signals.map (view count) count)
    |> mapError (\error: dom.DomRuntimeError<Never> -> "render failed")
}
```

### エディタとコンパイラが同じ答えを出す

CLI、エディタ用の LSP、フォーマッタ、ブラウザの Playground は、すべて同じコンパイラ本体を使っています。
エディタに表示される型やエラーと、実際にビルドしたときの結果が食い違いません。

## 試してみる

インストールせずに、ブラウザからすぐ試せます。

- **[Playground](https://seseragi.vercel.app/)** — コードを書いて、型チェック・実行・HTML のプレビューまでできます
- **[A Tour of Seseragi](https://seseragi.vercel.app/tour/)** — Hello world から Effect、Signal、Web UI までを順番に学べます
- **[サンプル集](./examples/samples/README.md)** — 現在のコンパイラで実際に動くサンプルです

## インストール

[GitHub Releases](https://github.com/KentaroMorishita/seseragi/releases) から、
OS と CPU に合ったアーカイブをダウンロードしてください。CLI と LSP が入っています。
生成されたプログラムの実行には [Bun](https://bun.sh/) 1.4.4 以降が必要です。

```sh
seseragi run hello.ssrg          # ファイルを実行する
seseragi new web my-app          # Web アプリのプロジェクトを作る
seseragi dev my-app              # 開発サーバーを起動する
seseragi build hello.ssrg        # ビルドする
seseragi format hello.ssrg       # 整形する
```

Web アプリを作るなら [はじめての Web アプリ](./docs/GETTING_STARTED.md) へ。
VS Code 拡張（ID: `seseragi-dev.seseragi`）は、同じ GitHub Releases にある VSIX からインストールできます。
詳しくは [Seseragi for VS Code](./extensions/seseragi/README.md) を参照してください。

> [!NOTE]
> Bun 1.4.3 以前には脆弱性（CVE-2026-111497）があります。
> Node.js で実行する場合も、2026年10月のセキュリティ修正を含むバージョン
> （22.23.4 / 24.22.1 / 26.12.1 以降）を使ってください。

## 開発状況

- TypeScript / JavaScript を生成するターゲットでは、ここに載せた機能はすべて動きます
- Rust のネイティブバイナリを生成するターゲットを開発中です。TypeScript ターゲットと同じ機能を使えるようにする作業を進めています（進捗: [#809](https://github.com/KentaroMorishita/seseragi/issues/809)）
- TypeScript との相互運用、ファイル・HTTP、Stream などは未実装か、一部だけ実装されています

どこまで実装済みかは [STATUS](./docs/STATUS.md)、今後の予定は [ROADMAP](./docs/ROADMAP.md) を参照してください。
仕様に書かれていても、まだ動かない機能があります。

## コンパイラの開発に参加する

コンパイラは Rust で書かれています。ソースからビルドするには Rust と Bun が必要です
（Rust のバージョンは `rust-toolchain.toml` で固定されています）。

```sh
# main ブランチから CLI をインストールする（リリース版ではありません）
cargo install --git https://github.com/KentaroMorishita/seseragi --locked seseragi-cli

# Playground をローカルで起動する
bun run dev:playground

# 変更した範囲に応じてチェックする（check:sample / check:playground / check:extension / check:release もあります）
bun run check:rust
```

> [!NOTE]
> rustup は 1.29.2 以降を使ってください（それより前のバージョンには証明書検証の脆弱性 CVE-2026-111943 があります）。

どのチェックを使うかは [Scoped checks](./docs/SCOPED_CHECKS.md)、
コンパイラの内部構成は [IMPLEMENTATION](./docs/IMPLEMENTATION.md) にまとめています。

| パス | 内容 |
| --- | --- |
| `crates/` | コンパイラ、CLI、LSP、WASM、テストランナー |
| `runtime/ts/` | 生成された TypeScript が使うランタイム |
| `apps/playground/` | Playground と Tour |
| `examples/samples/` | 動作するサンプル集 |
| `examples/tour/` | Tour の各レッスン |
| `examples/spec/` | 言語仕様のテストケース |
| `docs/spec/` | 言語仕様 |
| `extensions/seseragi/` | VS Code 拡張 |

## ライセンス

Apache-2.0

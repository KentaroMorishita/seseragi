<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./assets/brand/extension/logo-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="./assets/brand/extension/logo-light.svg">
    <img src="./assets/brand/extension/logo-light.svg" alt="Seseragi" width="720">
  </picture>
</p>

<p align="center"><strong>データの形から状態、UIまでを、ひとつの言語として組み立てる。</strong></p>

<p align="center">
  <a href="https://seseragi-docs.vercel.app/">ドキュメント</a>
  · <a href="https://seseragi.vercel.app/">Playground</a>
  · <a href="https://seseragi.vercel.app/tour/">Tour</a>
  · <a href="./extensions/seseragi/README.md">VS Code</a>
</p>

## 複雑なアプリを、美しいまま書く。

アプリは育つほど、コードが濁っていきます。
状態が増え、例外があちこちから飛んでくる。ただの計算に見える関数が、裏で通信している。
いつのまにか、コードを読んでも、何が起きるのか分からなくなっていきます。

Seseragi は、それに抗うための言語です。
複雑なアプリを、どこまで美しいコードのまま書けるか。それを本気で突き詰めています。

目指すのは、せせらぎのように澄んだコードです。
データは上流から下流へ、パイプラインを通って流れていく。どこで形を変え、どこで失敗しうるのか、水底まで見通せる。
そのために、Seseragi は次のことを約束します。

- **危険を隠さない** — `null` も `undefined` も例外もありません。欠けるかもしれない値は `Maybe`、失敗するかもしれない処理は `Either` や `Effect` として、型の上にはっきり現れます
- **魔法を使わない** — 型推論も暗黙の選択も、コンパイラが理由を一つに説明できるときだけ成功します。迷うときは、勝手に選ばずにエラーで教えてくれます
- **依存は推論し、失敗は設計する** — 関数が使うサービス（コンソール、時計、HTTP など）はコンパイラが集めて型にします。どんな失敗がありうるかは、あなたが型として設計します
- **抽象化しても、読めるまま** — 目指すのは短さではありません。データの形と処理の流れが一目で読めることです

Seseragi のコードは TypeScript にコンパイルされ、Bun、Node.js、ブラウザで動きます。
CLI ツールも Web アプリの画面も、同じ言語、同じ考え方で書けます。

> [!NOTE]
> Seseragi は開発中の実験的な言語です。構文や仕様は今後も変わる可能性があります。

## TypeScript で、こんなことはありませんか

- 読み込み中・成功・失敗の状態を管理していたら、画面に「Hello, undefined」と表示されてしまった
- ある関数がエラーを投げるのかどうか、型を見ても分からない。`catch` で受け取る値の型も `unknown` で、中身は実行してみるまで分からない
- ただの計算に見える関数が、裏で通信やログ出力をしていた

Seseragi では、どれもコンパイラが実行前に見つけます。順番に見ていきます。

### ありえない状態と、場合分けの漏れ

React などでデータを読み込むとき、状態をこんなふうに書くことがよくあります。

```ts
type UserState = {
  isLoading: boolean
  user?: { name: string }
  error?: string
}

function message(state: UserState): string {
  if (state.isLoading) return "Loading..."
  if (state.error) return `Error: ${state.error}`
  return `Hello, ${state.user?.name}`
}
```

この型は、「読み込み中なのにエラーもある」のような、ありえない組み合わせも許してしまいます。
`{ isLoading: false }` を渡すと `Hello, undefined` と表示されますが、strict モードでもコンパイルは通ります。

Seseragi では、状態を「どれか一つ」として型に書き出します。

```seseragi
type UserState =
  | Loading
  | Loaded String
  | Failed String

fn message state: UserState -> String =
  match state {
    Loading -> "Loading..."
    Loaded name -> `Hello, ${name}`
    Failed reason -> `Error: ${reason}`
  }
```

名前があるのは `Loaded` のときだけなので、`undefined` を表示してしまうことはありません。
さらに `match` は、すべての状態を扱っているかを必ずチェックします。新しい状態を足したのに処理を書き忘れると、ビルドする前にエラーになります。

```text
error[SES-T0301]: This match does not cover every possible value
  |   match state {
  |   ^ missing patterns: Failed _
```

[▶ Playground で開く](https://seseragi.vercel.app/?source=type%20UserState%20%3D%0A%20%20%7C%20Loading%0A%20%20%7C%20Loaded%20String%0A%20%20%7C%20Failed%20String%0A%0Afn%20message%20state%3A%20UserState%20-%3E%20String%20%3D%0A%20%20match%20state%20%7B%0A%20%20%20%20Loading%20-%3E%20%22Loading...%22%0A%20%20%20%20Loaded%20name%20-%3E%20%60Hello%2C%20%24%7Bname%7D%60%0A%20%20%20%20Failed%20reason%20-%3E%20%60Error%3A%20%24%7Breason%7D%60%0A%20%20%7D%0A%0Apub%20effect%20fn%20main%20%3D%20do%20%7B%0A%20%20Loading%20%7C%3E%20message%20%7C%3E%20println%0A%20%20Loaded%20%22Aoi%22%20%7C%3E%20message%20%7C%3E%20println%0A%20%20Failed%20%22timeout%22%20%7C%3E%20message%20%7C%3E%20println%0A%7D%0A)

### 例外のかわりに、失敗を型で表す

TypeScript では、関数が失敗するかどうかはシグネチャに現れません。

```ts
function parseNumber(text: string): number {
  const value = Number(text)
  if (Number.isNaN(value)) throw new Error(`not a number: ${text}`)
  return value
}
```

Seseragi では、失敗するかもしれない処理は `Either<エラーの型, 成功したときの型>` を返します。
成功なら `Right 値`、失敗なら `Left エラー` です。呼ぶ側は、型を見れば失敗しうることが分かります。

失敗しうる処理を続けて呼ぶときは `do` 記法を使います。`<-` は「成功していれば中の値を取り出す」という意味で、
どこかで `Left` が返ると、そこで止まってその `Left` が全体の結果になります。`try/catch` を書かずに、成功する流れだけを上から順に書けます。

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

[▶ Playground で開く](https://seseragi.vercel.app/?source=import%20%2A%20as%20either%20from%20%22std%2Feither%22%0Aimport%20%2A%20as%20int%20from%20%22std%2Fint%22%0A%0Afn%20parseNumber%20text%3A%20String%20-%3E%20Either%3CString%2C%20Int%3E%20%3D%0A%20%20int.parse%20text%20%7C%3E%20either.mapLeft%20%28%5C_%20-%3E%20%60not%20a%20number%3A%20%24%7Btext%7D%60%29%0A%0Afn%20divide%20total%3A%20Int%20-%3E%20count%3A%20Int%20-%3E%20Either%3CString%2C%20Int%3E%20%3D%20match%20count%20%7B%0A%20%200%20-%3E%20Left%20%22cannot%20divide%20by%20zero%22%0A%20%20_%20-%3E%20Right%20%28total%20%2F%20count%29%0A%7D%0A%0A%2F%2F%20%E3%81%A9%E3%81%93%E3%81%8B%E3%81%A7%20Left%20%E3%81%AB%E3%81%AA%E3%81%A3%E3%81%9F%E3%82%89%E3%80%81%E3%81%9D%E3%81%93%E3%81%A7%E6%AD%A2%E3%81%BE%E3%81%A3%E3%81%A6%E3%81%9D%E3%81%AE%20Left%20%E3%81%8C%E7%B5%90%E6%9E%9C%E3%81%AB%E3%81%AA%E3%82%8B%0Afn%20calculate%20left%3A%20String%20-%3E%20right%3A%20String%20-%3E%20Either%3CString%2C%20Int%3E%20%3D%20do%20%7B%0A%20%20total%20%3C-%20parseNumber%20left%0A%20%20count%20%3C-%20parseNumber%20right%0A%20%20divide%20total%20count%0A%7D%0A%0Apub%20effect%20fn%20main%20%3D%20do%20%7B%0A%20%20println%20%24%20show%20%28calculate%20%2284%22%20%222%22%29%0A%20%20println%20%24%20show%20%28calculate%20%2284%22%20%22zero%22%29%0A%20%20println%20%24%20show%20%28calculate%20%2284%22%20%220%22%29%0A%7D%0A)

### 副作用が型に出る

画面への出力や通信のような副作用は、`effect fn` の中でしか起こせません。
ふつうの `fn` で `println` を呼ぶと、型エラーになります。

```seseragi
fn greet name: String -> Unit = println `Hello, ${name}`
```

```text
error[SES-T0101]: Expression type does not match the expected type
  = expected: Unit
  = actual: Effect<{ console: Console }, ConsoleError, Unit>
```

`Effect<{ console: Console }, ConsoleError, Unit>` は、
「コンソールを使い、`ConsoleError` で失敗することがあり、成功すると `Unit` を返す処理」という意味です。
何に依存し、どう失敗するかが、型を見るだけで分かります。

`effect fn` の失敗の種類は、自分で型として設計して `fails` で宣言します。

```seseragi
type OrderError =
  | OutOfStock String
  | TooMany Int

effect fn reserve item: String -> quantity: Int -> String fails OrderError =
  match item {
    "tea" -> fail $ OutOfStock item
    _ -> if quantity > 10 then fail $ TooMany quantity else succeed `${quantity} x ${item}`
  }
```

[▶ Playground で開く](https://seseragi.vercel.app/?source=import%20%2A%20as%20effects%20from%20%22std%2Feffect%22%0A%0Atype%20OrderError%20%3D%0A%20%20%7C%20OutOfStock%20String%0A%20%20%7C%20TooMany%20Int%0A%0Aeffect%20fn%20reserve%20item%3A%20String%20-%3E%20quantity%3A%20Int%20-%3E%20String%20fails%20OrderError%20%3D%0A%20%20match%20item%20%7B%0A%20%20%20%20%22tea%22%20-%3E%20fail%20%24%20OutOfStock%20item%0A%20%20%20%20_%20-%3E%20if%20quantity%20%3E%2010%20then%20fail%20%24%20TooMany%20quantity%20else%20succeed%20%60%24%7Bquantity%7D%20x%20%24%7Bitem%7D%60%0A%20%20%7D%0A%0Afn%20explain%20error%3A%20OrderError%20-%3E%20String%20%3D%20match%20error%20%7B%0A%20%20OutOfStock%20item%20-%3E%20%60%24%7Bitem%7D%20is%20out%20of%20stock%60%0A%20%20TooMany%20quantity%20-%3E%20%60cannot%20reserve%20%24%7Bquantity%7D%20at%20once%60%0A%7D%0A%0Aeffect%20fn%20order%20item%3A%20String%20-%3E%20quantity%3A%20Int%20-%3E%20String%20fails%20Never%20%3D%0A%20%20reserve%20item%20quantity%0A%20%20%20%20%7C%3E%20effects.recover%20%28%5Cerror%20-%3E%20succeed%20%24%20explain%20error%29%0A%0Apub%20effect%20fn%20main%20%3D%20do%20%7B%0A%20%20coffee%20%3C-%20order%20%22coffee%22%202%0A%20%20println%20coffee%0A%20%20tea%20%3C-%20order%20%22tea%22%201%0A%20%20println%20tea%0A%7D%0A)

## ほかの特徴

### パイプライン

`|>` で値を次の関数へ渡していきます。
関数はカリー化されているので、引数の一部だけを渡した関数をそのままパイプラインに置けます。

```seseragi
import * as arrays from "std/array"

fn double n: Int -> Int = n * 2

pub effect fn main =
  [1, 2, 3, 4]
    |> arrays.filter (\n: Int -> n > 2)
    |> map double
    |> sum
    |> show
    |> println
// => 14
```

### UI も同じ言語で

画面も、ここまでと同じ型と関数で書きます。
状態は `Signal` に持たせ、`signals.map` で状態から画面を作ります。
ボタンを押して状態が変わると、画面も自動で更新されます。React も JSX も要りません。

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

[▶ Playground で開く](https://seseragi.vercel.app/?source=import%20%2A%20as%20dom%20from%20%22std%2Fweb%2Fdom%22%0Aimport%20%2A%20as%20html%20from%20%22std%2Fweb%2Fhtml%22%0Aimport%20%2A%20as%20signals%20from%20%22std%2Fsignal%22%0Aimport%20%7B%20MutableSignal%20%7D%20from%20%22std%2Fsignal%22%0A%0Afn%20label%20current%3A%20Int%20-%3E%20html.Html%3CTask%3CUnit%3E%3E%20%3D%20html.p%20%7B%20children%3A%20%60Count%3A%20%24%7Bcurrent%7D%60%20%7D%0A%0Afn%20view%20count%3A%20MutableSignal%3CInt%3E%20-%3E%20current%3A%20Int%20-%3E%20html.Html%3CTask%3CUnit%3E%3E%20%3D%0A%20%20html.div%20%7B%0A%20%20%20%20children%3A%20%5B%0A%20%20%20%20%20%20label%20current%2C%0A%20%20%20%20%20%20html.button%20%7B%0A%20%20%20%20%20%20%20%20onClick%3A%20signals.update%20%28%5Cn%3A%20Int%20-%3E%20n%20%2B%201%29%20count%2C%0A%20%20%20%20%20%20%20%20children%3A%20%22%2B1%22%0A%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20html.button%20%7B%20onClick%3A%20signals.set%200%20count%2C%20children%3A%20%22Reset%22%20%7D%0A%20%20%20%20%5D%0A%20%20%7D%0A%0Afn%20perform%20action%3A%20Task%3CUnit%3E%20-%3E%20Task%3CUnit%3E%20%3D%20action%0A%0Apub%20effect%20fn%20main%20-%3E%20Unit%20with%20dom%3A%20dom.Dom%20fails%20String%20%3D%20do%20%7B%0A%20%20count%20%3C-%20signals.make%200%0A%20%20target%20%3C-%20dom.query%20%22%23app%22%0A%20%20%20%20%7C%3E%20mapError%20%28%5Cerror%3A%20dom.DomError%20-%3E%20%22%23app%20not%20found%22%29%0A%20%20dom.run%20%28dom.defaultOptions%20%28%29%29%20target%20perform%20%28signals.map%20%28view%20count%29%20count%29%0A%20%20%20%20%7C%3E%20mapError%20%28%5Cerror%3A%20dom.DomRuntimeError%3CNever%3E%20-%3E%20%22render%20failed%22%29%0A%7D%0A)

### エディタとコンパイラが同じ答えを出す

CLI、エディタ用の LSP、フォーマッタ、ブラウザの Playground は、すべて同じコンパイラ本体を使っています。
エディタに表示される型やエラーと、実際にビルドしたときの結果が食い違いません。

### 生成されるのは普通の TypeScript

Seseragi のコードは TypeScript にコンパイルされ、Bun・Node.js・ブラウザで動きます。
たとえば上の `type UserState` は、`tag` プロパティで状態を見分けるユニオン型になります。

```ts
type UserState =
  | { readonly tag: "Loading" }
  | { readonly tag: "Loaded"; readonly value: string }
  | { readonly tag: "Failed"; readonly value: string };
```

TypeScript の既存のコードや npm パッケージを Seseragi から呼ぶ仕組みは、まだ開発中です（[開発状況](#開発状況)）。

## TypeScript との対応表

| TypeScript | Seseragi |
| --- | --- |
| `const total = 1` | `let total = 1` |
| `function add(a: number, b: number): number { return a + b }` | `fn add a: Int -> b: Int -> Int = a + b` |
| `(x) => x + 1` | `\x -> x + 1` |
| `format(parse(text))` | `text \|> parse \|> format` |
| `{ kind: "a" } \| { kind: "b"; value: number }` | `type T = \| A \| B Int` |
| `switch (value.kind) { ... }` | `match value { ... }` |
| `T \| undefined` | `Maybe<T>`（`Just value` / `Nothing`） |
| `throw` / `try ... catch` | `Either<E, A>`、`effect fn ... fails E` |
| `async function` | `effect fn`（非同期処理も Effect で表します） |
| `interface` と、その実装 | `trait` と `instance` |
| `{ ...user, name: "Aoi" }` | `{ ...user, name: "Aoi" }` |
| `` `Hello, ${name}` `` | `` `Hello, ${name}` `` |

関数、データ、失敗、非同期処理、React との違いまで、TypeScript と並べた詳しい比較は
ドキュメントの [TypeScript から Seseragi へ](https://seseragi-docs.vercel.app/docs/guides/typescript-to-seseragi) にあります。

## 試してみる

インストールせずに、ブラウザからすぐ試せます。

- **[ドキュメント](https://seseragi-docs.vercel.app/)** — 言語を順番に学ぶ「学ぶ」、目的別の「用途別ガイド」、規則を調べる「リファレンス」があります
- **[Playground](https://seseragi.vercel.app/)** — コードを書いて、型チェック・実行・HTML のプレビューまでできます
- **[A Tour of Seseragi](https://seseragi.vercel.app/tour/)** — Hello world から Effect、Signal、Web UI までを順番に学べます
- **[サンプル集](./examples/samples/README.md)** — 現在のコンパイラで実際に動くサンプルです

たとえば、FizzBuzz はこう書けます。

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

詳しい手順はドキュメントの [ローカルで使う](https://seseragi-docs.vercel.app/docs/learn/local-development)、Web アプリを作るなら [Web アプリを作る](https://seseragi-docs.vercel.app/docs/guides/web-app) を参照してください。
VS Code 拡張（ID: `seseragi-dev.seseragi`）は、同じ GitHub Releases にある VSIX からインストールできます。
詳しくは [Seseragi for VS Code](./extensions/seseragi/README.md) を参照してください。

> [!NOTE]
> Bun 1.4.3 以前には脆弱性（CVE-2026-111497）があります。
> Node.js で実行する場合も、2026年10月のセキュリティ修正を含むバージョン
> （22.23.4 / 24.22.1 / 26.12.1 以降）を使ってください。

## 開発状況

- TypeScript を生成するターゲットでは、この README に載せた機能はすべて動きます
- TypeScript の既存コードや npm パッケージとの相互運用は開発中です
- ファイル・HTTP・Stream などの標準ライブラリは、一部だけ実装されています
- Rust のネイティブバイナリを生成するターゲットも開発中です（進捗: [#809](https://github.com/KentaroMorishita/seseragi/issues/809)）

詳しくは [STATUS](./docs/STATUS.md)（今動くもの）と [ROADMAP](./docs/ROADMAP.md)（これからの予定）を参照してください。

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

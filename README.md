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

> [!IMPORTANT]
> Seseragi は開発中の実験的な言語です。構文・型・ランタイムの仕様は今後も変わる可能性があり、
> 本番環境での利用はまだ想定していません。

# Seseragi

Seseragi は、データの定義から値の変換、状態の管理、UI の構築までを、
ひとつの型システムと構文で書けるプログラミング言語です。
パターンマッチ、パイプライン、ジェネリクスを中心に、
「データがどんな形をしていて、どう流れていくか」がコードから素直に読めることを大切にしています。

コンパイラは Rust で書かれていて、生成先は TypeScript / JavaScript です。
Rust のネイティブバイナリを生成するターゲットも開発中です。

## コードの例

[`examples/samples/fizzbuzz/main.ssrg`](./examples/samples/fizzbuzz/main.ssrg) より:

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
14
FizzBuzz
...
```

3 と 5 で割った余りをタプルにまとめ、ひとつの `match` で場合分けしています。
`for` で 1 から 30 までを順に処理し、`println` で結果を出力します。

## 書けるもの

| 分野 | 内容 |
| --- | --- |
| データと型 | ADT、struct、record、ジェネリクス、パターンマッチ、newtype、型エイリアス |
| 関数の合成 | 関数、パイプライン、独自の演算子、Functor / Applicative / Monad |
| 副作用 | `effect fn`、`do` 記法、型付きのエラー、標準入出力 |
| リアクティブ UI | `Signal`、HTML の記述、関数コンポーネント、DOM イベント、フォーム |
| 開発ツール | CLI、フォーマッタ、LSP、VS Code 拡張、ブラウザで動く Playground |

どこまで実装済みかは [STATUS](./docs/STATUS.md) にまとめています。
仕様に書かれていても、まだ動かない機能があります。

## 試してみる

インストールせずに試すなら、ブラウザから使えます。

- **[Playground](https://seseragi.vercel.app/)** — コードを書いて、型チェック・実行・HTML のプレビューまでできます
- **[A Tour of Seseragi](https://seseragi.vercel.app/tour/)** — Hello world から Effect、Signal、Web UI までを順番に学べます
- **[サンプル集](./examples/samples/README.md)** — 現在のコンパイラで実際に動くサンプルです

## インストール

[GitHub Releases](https://github.com/KentaroMorishita/seseragi/releases) から、
OS と CPU に合ったアーカイブをダウンロードしてください。CLI と LSP が入っています。

生成されたプログラムの実行には [Bun](https://bun.sh/) 1.4.4 以降が必要です（Rust は不要です）。

> [!NOTE]
> Bun 1.4.3 以前には脆弱性（CVE-2026-111497）があります。
> Node.js で実行する場合も、2026年10月のセキュリティ修正を含むバージョン
> （22.23.4 / 24.22.1 / 26.12.1 以降）を使ってください。

インストールできたら、次のように使います。

```sh
seseragi --version

# ファイルを実行する
seseragi run hello.ssrg

# Web アプリのプロジェクトを作って、開発サーバーを起動する
seseragi new web my-app
seseragi dev my-app

# ビルド・整形・テスト
seseragi build hello.ssrg --out-dir dist
seseragi format hello.ssrg
seseragi test my-app
```

Web アプリを作る場合は [Getting Started](./docs/GETTING_STARTED.md) に、
プロジェクトの作成から VS Code の設定、本番ビルドまでの手順があります。
配布物の構成やチェックサムについては [リリース手順](./docs/RELEASE.md) を参照してください。

## VS Code

拡張機能の ID は `seseragi-dev.seseragi` です。
GitHub Releases から OS と CPU に合った VSIX をダウンロードし、
コマンドパレットの `Extensions: Install from VSIX...` でインストールします。
LSP は VSIX に同梱されています。

設定やフォーマッタについては [Seseragi for VS Code](./extensions/seseragi/README.md) を参照してください。

## 開発状況

- コンパイラ、CLI、LSP、フォーマッタ、Playground は、ひとつのコンパイラ本体を共有しています。エディタで見える型と、実際にコンパイルされる結果が食い違いません
- TypeScript ターゲットでは、ADT、パターンマッチ、Effect、Signal、Web UI がサンプルとテストで動作を確認できています
- [Rust ネイティブターゲット](./docs/architecture/rust-backend.md)は、TypeScript ターゲットと同じ機能を使えるようにする作業を進めています（進捗: [#809](https://github.com/KentaroMorishita/seseragi/issues/809)）
- TypeScript との相互運用、ファイル・HTTP、Stream などは未実装か、一部だけ実装されています

詳しくは [STATUS](./docs/STATUS.md)（現状）、[ROADMAP](./docs/ROADMAP.md)（今後の予定）、
[SPEC COVERAGE](./docs/SPEC_COVERAGE.md)（仕様のテスト状況）を参照してください。

## コンパイラの開発に参加する

ソースからビルドするには Rust と Bun が必要です。
Rust のバージョンは `rust-toolchain.toml` で固定されています。

> [!NOTE]
> rustup は 1.29.2 以降を使ってください（それより前のバージョンには証明書検証の脆弱性 CVE-2026-111943 があります）。
> Homebrew で入れた場合は `brew upgrade rustup` で更新できます。

```sh
# main ブランチから CLI をインストールする（リリース版ではありません）
cargo install \
  --git https://github.com/KentaroMorishita/seseragi \
  --locked \
  seseragi-cli
```

よく使うコマンド:

```sh
# Playground をローカルで起動する
bun run dev:playground

# 変更した範囲に応じてチェックする
bun run check:sample
bun run check:playground
bun run check:rust
bun run check:extension
bun run check:release

# 複数の範囲にまたがる変更のときだけ、全体をチェックする
bun run check

# コンパイラやランタイムを変更したときに、Playground の WASM を作り直す
bun run build:playground:wasm
```

どのチェックを使うかは [Scoped checks](./docs/SCOPED_CHECKS.md) に、
コンパイラ内部の構成は [IMPLEMENTATION](./docs/IMPLEMENTATION.md) にまとめています。

### ディレクトリ構成

| パス | 内容 |
| --- | --- |
| `crates/` | Rust 製のコンパイラ、CLI、LSP、WASM、テストランナー |
| `runtime/ts/` | 生成された TypeScript が使うランタイム |
| `apps/playground/` | Playground と Tour |
| `examples/samples/` | 動作するサンプル集（Playground の Discover にも表示されます） |
| `examples/tour/` | Tour の各レッスン |
| `examples/spec/` | 言語仕様のテストケース |
| `docs/spec/` | 言語仕様 |
| `extensions/seseragi/` | VS Code 拡張 |

## ライセンス

Apache-2.0

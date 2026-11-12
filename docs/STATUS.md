# 開発状況

このページは、Seseragi のいまの実装状況を、コンパイラーの開発に参加する人向けにまとめたものです。
言語の使い方は[ドキュメントサイト](https://seseragi-docs.vercel.app/)、言語の決まりは[言語仕様](./spec/README.md)、
これからの予定は[ROADMAP](./ROADMAP.md)を見てください。

最終更新: 2026-11-12

## ひと目で

| 部分 | 状況 |
| --- | --- |
| 言語仕様 | 全章の初稿があります。実装しながら直しています |
| TypeScript ターゲット | 日常的に使える状態です。README とドキュメントサイトに載せた機能はすべて動きます |
| ネイティブターゲット（Rust） | 開発中です。公式サンプル31本のうち、画面を持たない24本すべてが動きます。サーバーと並行処理はこれからです |
| CLI | `run`・`build`・`dev`・`test`・`format`・`lint` などが使えます |
| エディター | VS Code 拡張と LSP があります |
| Playground | ブラウザで書いて実行できます。Tour もあります |

## TypeScript ターゲット

コンパイラーは Rust で書かれていて、TypeScript を生成します。生成したコードは Bun・Node.js・ブラウザで動きます。

使える主な機能は次のとおりです。

- ADT、struct、レコード、newtype、パターンマッチ（網羅性の検査つき）
- ジェネリクス、型クラス（trait と instance）、`deriving`
- `Maybe`・`Either`・Effect と `do` 記法、型で表した失敗
- Signal と、Signal を使った Web UI
- モジュール、パッケージ、ロックファイル
- TypeScript の関数を呼ぶ `foreign` と、`.d.ts` からの binding 生成

仕様10章の標準モジュールは、TypeScript ターゲットで使えます。仕様と実装の細かなずれは、[#1274](https://github.com/KentaroMorishita/seseragi/issues/1274) の子 Issue で追っています。

## ネイティブターゲット

TypeScript ターゲットと同じプログラムを、Bun や Node.js なしで動くネイティブバイナリにする作業を進めています。
全体の進み具合は [#809](https://github.com/KentaroMorishita/seseragi/issues/809) で追っています。

`seseragi run`・`build`・`dev` に `--target native` を付けると、ネイティブでビルドして実行できます。

ネイティブで使えるもの:

- 言語機能（M1）: 公式サンプルが使う機能すべて
- CLI ツールに要るもの（M2）: ファイル・プロセス・テキスト・正規表現・日時・テストなど

まだ足りないもの:

- サーバーと並行処理（M3）: Effect の service と provide、race・parallel、`std/queue`、`std/semaphore`、Stream の一部、HTTP、Server-Sent Events、WebSocket
- 外部との連携（M4）: `foreign` 宣言、provider のネイティブ実装、SQLite と Postgres の adapter

それぞれの Issue は、[ROADMAP](./ROADMAP.md) のマイルストーンから探せます。

### 公式サンプルの状況

`examples/samples` の31本を、ネイティブでビルドして、期待する出力と比べた結果です。

| 状況 | 本数 | 内容 |
| --- | --- | --- |
| 動く（TS と出力が一致） | 24 | 画面を持たないサンプルすべて。`std/web/html` で HTML を文字列にする html-components を含みます |
| ブラウザー専用（ネイティブの対象外） | 7 | `std/web/dom` で画面を動かすサンプル。TypeScript ターゲットが担当します |

### TS とネイティブの一致を確かめる方法

- `examples/spec/fixtures/rust-backend/` の fixture は、Bun・Node.js・ネイティブの出力を、同じ期待値と比べています
- 型の付いたプログラムをランダムに作って両方で実行し、結果を比べる差分テストがあります（`crates/seseragi-cli/tests/differential.rs`）

## 開発用ツール

| ツール | 状況 |
| --- | --- |
| CLI | `new`・`run`・`build`・`dev`・`lock update`・`test`・`benchmark`・`format`・`lint`・`doc --test`・`dts convert` |
| LSP | 同じコンパイラー本体で、エディター上にエラーを表示します |
| フォーマッター | `seseragi format` で整形できます。スタイルの設定はありません |
| VS Code 拡張 | GitHub Releases の VSIX からインストールできます。LSP を同梱しています |
| Playground | コンパイラーを WASM にしてブラウザで動かしています。Tour と公式サンプルを開けます |

## 関連する文書

- [IMPLEMENTATION](./IMPLEMENTATION.md): コンパイラーの内部構成
- [変更に合わせて実行するチェック](./SCOPED_CHECKS.md): 変更に応じてどのチェックを実行するか
- [RELEASE](./RELEASE.md): リリースの手順
- [SPEC_COVERAGE](./SPEC_COVERAGE.md): 仕様で決まっている機能と、採用しないと決めた機能の一覧

## このページの直し方

マイルストーンを閉じたとき、または公式サンプルの数や動く本数が変わったときに直します。サンプルの本数は、`seseragi run --target native` で実際に実行した結果から書きます。

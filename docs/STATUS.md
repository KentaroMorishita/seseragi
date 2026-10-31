# 開発状況

このページは、Seseragi のいまの実装状況を、コンパイラーの開発に参加する人向けにまとめたものです。
言語の使い方は[ドキュメントサイト](https://seseragi-docs.vercel.app/)、言語の決まりは[言語仕様](./spec/README.md)、
これからの予定は[ROADMAP](./ROADMAP.md)を見てください。

最終更新: 2026-10-31

## ひと目で

| 部分 | 状況 |
| --- | --- |
| 言語仕様 | 全章の初稿があります。実装しながら直しています |
| TypeScript ターゲット | 日常的に使える状態です。README とドキュメントサイトに載せた機能はすべて動きます |
| ネイティブターゲット（Rust） | 開発中です。Web UI 以外の公式サンプル23本のうち16本が動きます |
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

まだ足りないものは次のとおりです。

- 標準ライブラリのうち、ファイル・HTTP・Stream などは一部だけです
- 省略可能なフィールドのパターン（`{ id? }`）は型検査が未対応です（#1174）
- 多相なトップレベル値を複数の型で使うと、生成したコードが読み込めません（#1173）

## ネイティブターゲット

TypeScript ターゲットと同じプログラムを、Bun や Node.js なしで動くネイティブバイナリにする作業を進めています。
全体の進み具合は [#809](https://github.com/KentaroMorishita/seseragi/issues/809) で追っています。

いまは `seseragi experimental build-native` でビルドします。`seseragi run --target native` で選べるようにする作業は #1122 です。

### 公式サンプルの状況

`examples/samples` の31本を、ネイティブでビルドして TypeScript ターゲットと出力を比べた結果です。

| 状況 | 本数 | 内容 |
| --- | --- | --- |
| 動く（TS と出力が一致） | 16 | fizzbuzz、records、stdin-greeting、project-greeting など |
| Web UI（ネイティブの対象外） | 8 | `std/web/html` を使うサンプル。画面は TypeScript ターゲットが担当します |
| まだ動かない | 7 | 下の表のとおり |

| 止まる理由 | サンプル | Issue |
| --- | --- | --- |
| `std/signal` が未対応 | signal-state、signal-composition、type-aliases | #1076 |
| 内包表記が未対応 | collections、typeclass-operator-parity | #1096、#1097 |
| Effect 以外の `do` が未対応 | either-and-monad | #1103 |
| パターンの判定の一部が未対応 | collection-patterns | #1093 |

### TS とネイティブの一致を確かめる方法

- `examples/spec/fixtures/rust-backend/` の fixture は、Bun・Node.js・ネイティブの出力を、同じ期待値と比べています
- 型の付いたプログラムをランダムに作って両方で実行し、結果を比べる差分テストを準備しています（#1170）

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
- [Scoped checks](./SCOPED_CHECKS.md): 変更に応じてどのチェックを実行するか
- [RELEASE](./RELEASE.md): リリースの手順
- [SPEC_COVERAGE](./SPEC_COVERAGE.md): 仕様と fixture の対応表

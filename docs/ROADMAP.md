# ロードマップ

このページは、Seseragi のこれからの予定をまとめたものです。いまの状況は [STATUS](./STATUS.md) を見てください。

最終更新: 2026-10-31

## いまの目標

TypeScript ターゲットで使える言語機能と標準ライブラリを、ネイティブターゲットでも同じように使えるようにします。
生成したネイティブバイナリは、Bun・Node.js・TypeScript のランタイムなしで動きます。利用者に Rust の所有権のような注釈を書かせることはしません。

ブラウザの UI は、これからも TypeScript ターゲットが担当します。ネイティブで GUI を作る予定はありません。

全体の進み具合は [#809](https://github.com/KentaroMorishita/seseragi/issues/809) で追っています。

## マイルストーン

| マイルストーン | ゴール |
| --- | --- |
| M1 言語機能と公式サンプル | Web UI 以外の公式サンプルがすべてネイティブで動き、`seseragi run --target native` が使える |
| M2 CLI ツールが書ける | ファイル・プロセス・テキスト・正規表現・日時・テストなどがネイティブで使える |
| M3 サーバーと並行処理 | HTTP・WebSocket・Effect の並行処理・Stream がネイティブで使える |
| M4 WASM と外部連携 | program WASM の対応を広げ、`foreign`・Provider・データベースの adapter を使えるようにする |

マイルストーンごとの Issue は、GitHub の [Milestones](https://github.com/KentaroMorishita/seseragi/milestones) にあります。

### M1 の残り

公式サンプルを動かすために残っているもの:

- `std/signal`（#1076）
- Array と List の内包表記（#1096、#1097）
- Effect 以外の Monad での `do` 記法（#1103）
- Array の固定長パターンと rest パターン（#1093）

そのほかの M1 の作業:

- `seseragi run` / `build` / `dev` でネイティブターゲットを選べるようにする（#1122、#1125）
- 実行時エラーを元のソースの位置で表示する（#1124）
- 標準の instance と、辞書を後から渡す関数をそろえる（#1101、#1102、#1104）

## TypeScript ターゲットで続けること

ネイティブの作業と並行して、TypeScript ターゲットでも次を進めます。

- 省略可能なフィールドのパターン（#1174）
- 多相なトップレベル値の生成コードの修正（#1173）
- ファイル・HTTP・Stream など、標準ライブラリの残り
- TypeScript の既存コードや npm パッケージとの連携の拡充

## 進め方

- 機能ごとに、TypeScript とネイティブで同じ出力になることを fixture で確かめます
- 型の付いたプログラムをランダムに作って両方で実行する差分テストで、書いていない組み合わせのずれも探します（#1170）
- CLI・LSP・Playground は同じコンパイラー本体を使います。言語の機能を足したら、これらからも同じように使えるようにします

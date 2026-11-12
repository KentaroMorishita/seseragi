# ロードマップ

このページは、Seseragi のこれからの予定を、コンパイラーの開発に参加する人向けにまとめたものです。いまの状況は [STATUS](./STATUS.md) を見てください。

残っている作業の一覧は、ここには書きません。GitHub のマイルストーンと tracking の Issue が、いつも最新の一覧です。

## いまの目標

TypeScript ターゲットで使える言語機能と標準ライブラリを、ネイティブターゲットでも同じように使えるようにします。
生成したネイティブバイナリは、Bun・Node.js・TypeScript のランタイムなしで動きます。利用者に Rust の所有権のような注釈を書かせることはしません。

ブラウザの UI は、これからも TypeScript ターゲットが担当します。ネイティブで GUI を作る予定はありません。

全体の進み具合は [#809](https://github.com/KentaroMorishita/seseragi/issues/809) で追っています。

## マイルストーン

| マイルストーン | ゴール | 状況 |
| --- | --- | --- |
| [M1 言語機能と公式サンプル](https://github.com/KentaroMorishita/seseragi/milestone/1) | Web UI 以外の公式サンプルがすべてネイティブで動き、`seseragi run --target native` が使える | 個別の Issue は完了。tracking の Issue（#811、#817）が残っています |
| [M2 CLI ツールが書ける](https://github.com/KentaroMorishita/seseragi/milestone/2) | ファイル・プロセス・テキスト・正規表現・日時・テストなどがネイティブで使える | 完了 |
| [M3 サーバーと並行処理](https://github.com/KentaroMorishita/seseragi/milestone/3) | HTTP・WebSocket・Effect の並行処理・Stream がネイティブで使える | **いま進めているマイルストーン** |
| [M4 WASM と外部連携](https://github.com/KentaroMorishita/seseragi/milestone/4) | program WASM の対応を広げ、`foreign`・Provider・データベースの adapter を使えるようにする | M3 のあとに進めます |

次に着手できる Issue は、M3 のマイルストーンの一覧から選びます。M3 には、次の分野の Issue があります。

- Effect の service と provide、race・parallel・並行 traverse
- `std/queue`、`std/semaphore`
- Stream の作成・変換・終端操作と、zip・merge・buffer
- HTTP のクライアントとサーバー、multipart、Server-Sent Events、WebSocket

分野ごとの tracking の Issue:

| Issue | 分野 |
| --- | --- |
| [#811](https://github.com/KentaroMorishita/seseragi/issues/811) | 言語機能 |
| [#812](https://github.com/KentaroMorishita/seseragi/issues/812) | 標準モジュール |
| [#813](https://github.com/KentaroMorishita/seseragi/issues/813) | I/O・時刻・並行処理・provider |
| [#814](https://github.com/KentaroMorishita/seseragi/issues/814) | 外部ホストとの境目（`foreign`、callback、ハンドル） |
| [#815](https://github.com/KentaroMorishita/seseragi/issues/815) | program WASM と TypeScript の UI |
| [#817](https://github.com/KentaroMorishita/seseragi/issues/817) | CLI・LSP・開発の流れ |

## TypeScript ターゲットで続けること

ネイティブの作業と並行して、TypeScript ターゲットでも次を進めます。

- 点検で見つかった、診断・標準ライブラリ・文書の改善（[#1274](https://github.com/KentaroMorishita/seseragi/issues/1274)）
- TypeScript の既存コードや npm パッケージとの連携の拡充

## 進め方

- 機能ごとに、TypeScript とネイティブで同じ出力になることを fixture で確かめます
- 型の付いたプログラムをランダムに作って両方で実行する差分テスト（`crates/seseragi-cli/tests/differential.rs`）で、fixture に書いていない組み合わせのずれも探します
- CLI・LSP・Playground は同じコンパイラー本体を使います。言語の機能を足したら、これらからも同じように使えるようにします

## このページの直し方

マイルストーンを閉じたとき、または新しいマイルストーンを作ったときに、上の表を直します。Issue の番号を一覧として書き足すことはしません。

このサンプルを選ぶ理由: Playground で試したコードを、手元の Web プロジェクトとして作り、VS Code、`seseragi dev`、本番用のビルドへ持ち出す一番短い道を確かめたいときに選びます。

## 手元のプロジェクトを作る

インストールした CLI から、同じソースを作れます。ひな形を別に用意しているのではなく、このサンプルの `src/main.ssrg`、`src/app.ssrg`、`tests/app.ssrg` が、そのまま CLI に入っています。

```sh
seseragi new web hello-web
cd hello-web
seseragi dev --open
```

## ファイルの役割

- `src/app.ssrg`: 状態（`StarterState`）、操作（`StarterAction`）、状態を変える `update`、画面を作る `view` を持ちます。`update` は構造体の一部だけを変える書き方（`{ ...state, count: … }`）を使っているので、状態に項目を足しても直す場所が増えません。見た目は名前を付けた `html.Style` の関数にしています。
- `src/main.ssrg`: `#app` を探し、`create` が作った画面の Signal を `dom.run` に渡すだけです。
- `tests/app.ssrg`: 画面を使わずに `update` を確かめるテストです。`seseragi test` で動きます。

本番用の出力は、同じ設定（`seseragi.toml`）から作ります。

```sh
seseragi build .
```

一つのファイルで書く小さな形は `interactive-app`、機能ごとに状態を分ける構成は `project-flow-app` を見てください。

このアプリは `Model`（状態を表す `Counter`）、操作を表す `Action`、状態更新関数 `update`、表示関数 `view` を組み合わせて動きます。ボタンの `onClick` は型付きの `Action` を返し、`update` は `Action` と現在の `Counter` だけから次の状態を計算します。`view` は現在の状態を受け取って純粋な `Html` を返します。

`dom.app` は 1 つの画面単位を手軽に動かすための仕組みです。内部で `MutableSignal<Counter>` を管理し、クリック操作ごとに `update` の結果を `Task<Unit>` の更新として反映してから画面を再描画します。複数の部品がそれぞれ自分の状態を管理する構成では、各部品が `Html<Task<Unit>>` を親へ渡し、全体の入口で `dom.run` が各操作の `Task` を実行する形で連携できます。

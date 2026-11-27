このサンプルを選ぶ理由: 互いに関係する複数の Signal を、途中状態を見せずにまとめて更新する仕組みを確かめたいときに選びます。

珈琲豆の在庫（`stock`）と販売済み数（`sold`）を別々の `MutableSignal` に持ち、`signals.combine` でまとめた在庫レポートを `signals.transaction` で一括更新します。

## この画面で試すこと

- Try next の「午前の注文を 5 袋に増やす」を押し、在庫が減った分だけ販売済みが増えて累計が保たれるのを見る
- 「午後の補充を 10 袋に増やす」を押し、補充後の在庫と累計がまとめて変わるのを見る
- `signals.planUpdate` を `signals.planSet` に変えて、直接値を置き換えるトランザクションも試す

## コードの読みどころ

1. `signals.combine formatInventory stock sold`: 2 つの Signal から 1 つの導出状態 `summary` を作ります。元が変わると `*summary` は常に最新の組み合わせを返します。
2. `signals.transaction`: `planUpdate` の配列を 1 回のトランザクションとして確定するため、在庫だけ減って販売数がまだ増えていない途中状態は外から見えません。
3. `*summary`: 各時点の確定したスナップショットを読み取り、開店時・午前・午後の推移を並べて出力します。

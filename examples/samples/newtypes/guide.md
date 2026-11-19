このサンプルを選ぶ理由: 同じ数値で表せる値どうしの取り違えを型で防ぎ、金額計算を正確に行いたいときに選びます。

注文明細の金額（`Yen`）と個数（`Quantity`）を別々の `newtype` に分け、定義した `+`・`-`・`*` で合計します。割引や消費税の途中計算は `Decimal` で誤差なく扱い、1 円未満は `Floor`（切り捨て）で整数円に揃えます。

## この画面で試すこと

- Try next の「会員割引を 20% にする」を押し、割引額と消費税が整数円でどう変わるかを見る
- 「注文に商品を 1 つ足す」を押し、通常注文と会員割引の両方が更新されるのを見る
- `lineSubtotal` で `line.unitPrice + line.quantity` のように別の型どうしを足してみて、型エラーが出ることを確かめる

## コードの読みどころ

1. `newtype Yen` と `newtype Quantity`: 中身が数値でも型としては別物になり、引数の取り違えを防げます。`Invoice` の金額もすべて `Yen` で持ち、表示のときだけ `rawYen` で取り出します。
2. `impl Yen`: 内部の `Decimal` が持つ演算子は自動では引き継がれないため、`Yen + Yen`・`Yen - Yen`・`Yen * Quantity` だけを定義しています。
3. `percentOfYen`: `decimal.divideExact` で正確な十進数のまま割り算し、日本の消費税の端数処理に合わせて `decimal.quantize 0 Floor` で 1 円未満を切り捨てます。
4. `let memberOrder`: `Order { ...standardOrder, discountPercent: 15 }` により、明細や税率はそのままに割引率だけを変えた新しい注文を作ります。

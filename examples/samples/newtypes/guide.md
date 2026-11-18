このサンプルを選ぶ理由: 同じ数値で表せる値どうしの取り違えを型で防ぎ、金額計算を正確に行いたいときに選びます。

注文明細の金額（`Yen`）と個数（`Quantity`）を別々の `newtype` に分け、定義した `+` と `*` で合計します。割引や消費税の端数は `Decimal` で誤差なく扱い、最後に整数円へ丸めます。

## この画面で試すこと

- Try next の「会員割引を 20% にする」を押し、割引額と税の小数部分がどう変わるかを見る
- 「注文に商品を 1 つ足す」を押し、通常注文と会員割引の両方が更新されるのを見る
- `lineSubtotal` で `line.unitPrice + line.quantity` のように別の型どうしを足してみて、型エラーが出ることを確かめる

## コードの読みどころ

1. `newtype Yen` と `newtype Quantity`: 中身が数値でも型としては別物になり、引数の取り違えを防げます。
2. `impl Yen`: 内部の `Decimal` が持つ演算子は自動では引き継がれないため、`Yen + Yen` と `Yen * Quantity` だけを定義しています。
3. `buildInvoice`: `decimal.divideExact` で割引と税を正確な十進数のまま計算し、`decimal.quantize 0 HalfEven` で請求額を整数円へ丸めます。
4. `let memberOrder`: `Order { ...standardOrder, discountPercent: 15 }` により、明細や税率はそのままに割引率だけを変えた新しい注文を作ります。

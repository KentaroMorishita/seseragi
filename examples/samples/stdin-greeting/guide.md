まずRunを押して出力を確かめたら、Input欄の文字を変えたり空にしたりしてもう一度実行してみてください。

`readLine ()`は入力の終端を例外にせず`Maybe<String>`で返します。`Stdin`と`Console`のエラー型は`mapError`で`AppError`へ揃えています。

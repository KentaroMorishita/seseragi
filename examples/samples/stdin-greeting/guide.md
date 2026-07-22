`readLine ()`はEOFを例外にせず`Maybe<String>`で返します。StdinとConsoleの失敗は`mapError`で`AppError`へ揃えます。Stdin欄の値を変え、入力ありと空入力の扱いを試せます。

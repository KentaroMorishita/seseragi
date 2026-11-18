このサンプルを選ぶ理由: 状態を持たない画面を、普通の関数を部品にして組み立て、HTML の文字列として書き出す（SSR）形を見たいときに選びます。

今週読んだ三冊の本を、同じカードの部品で並べます。表紙は画像を読み込まず、`std/web/svg` で描いています。

コードは四つの区切りに分かれています。

- **データ:** 本は `Book` 構造体で、三冊を `books` の配列にまとめています。本を足すときは、この配列に一つ足すだけです。
- **見た目:** `html.Style` は値なので、`cardStyle` のように名前を付けて使い回せます。`shelfStyle` の `variables` は CSS のカスタムプロパティ（`--card-shadow`）になり、カードの影の値を一か所で決めています。カードの並びは `repeat(auto-fill, minmax(240px, 1fr))` なので、画面の幅に合わせて列の数が変わります。
- **部品:** `bookCard` や `tag` は、`Book` や文字列を受け取って `html.Html` を返す普通の関数です。`map bookCard books` で、配列からカードを並べます。子要素の配列には `Array<html.Html<Unit>>` と型を書いています。
- **実行の境界:** `main` で `html.renderToString` を使って HTML の文字列にし、`println` で出力します。Preview は、その文字列を表示しています。

操作に合わせて画面を変えたいときは `interactive-app`、複数のファイルに分けた形は `web-starter` を見てください。

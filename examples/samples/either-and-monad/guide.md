このサンプルを選ぶ理由: 入力のチェックで「最初の誤りで止める」か「全部の誤りを一覧で見せる」かを使い分けたいときに選びます。

会員登録の 4 つの項目（名前・メールアドレス・パスワード・年齢）を、同じ検査関数から `Either` の `do` 構文と `Validation` の `<*>` の両方に通して結果を比べます。

## この画面で試すこと

- Try next の「名前だけ先に直す」を押し、`Either` の停止箇所が次の項目（メールアドレス）へ移り、`Validation` は残りの 3 件を出すのを見る
- 続けて「メールアドレスも直す」を押し、`Either` がパスワードへ進み、`Validation` が残り 2 件になるのを見る
- `validForm` の `age` を `17` に変えて、誤りが 1 件だけのときは両方とも同じ 1 件を返すことを確かめる

## コードの読みどころ

1. `checkName` 〜 `checkAge`: 各項目の規則は `Either` を返す小さな純粋関数として一度だけ書きます。
2. `registerFailFast`: `do` 構文は前の行で `Left` が返ると残りの行を実行せず、最初の誤りだけを返します。
3. `registerCollectAll`: `pure makeMember <*> ...` に `validation.fromEither` を渡すと、互いに依存しない 4 つの検査をすべて実行し、見つかった誤りを順番に集めます。
4. `renderComparison`: `Invalid errors` が持つ空でない配列（`NonEmptyList`）を `arrays.fromIterable` で配列に直し、表示用の行へ組み立てます。

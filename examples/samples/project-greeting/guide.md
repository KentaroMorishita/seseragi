このサンプルを選ぶ理由: 計算ロジック、表示整形、エントリポイント、単体テストを別ファイルに分けたパッケージの構成を見たいときに選びます。

送料計算だけを担う `pricing/shipping.ssrg`、結果を表示用の文に整える `pricing/format.ssrg`、入出力をつなぐ `main.ssrg` に役割を分け、Explorer から切り替えて編集できます。テスト本体は `tests/shipping.ssrg` に分けて置いてあり、手元の CLI で `seseragi test` を実行すると単体テストが走ります。

## この画面で試すこと

- Try next の「珈琲豆を 5000 円にして送料無料を見る」を押し、国内配送が 5000 円以上で送料無料に切り替わるのを見る
- 「配送見積もりに箱物を 1 件足す」を押し、2kg 超えの重量追加料金（350 円）が加算されるのを見る
- `pricing/shipping.ssrg` の `baseShipping` を `main.ssrg` から直接呼ぼうとして、`pub` のない関数がモジュール外に漏れないことを確かめる

## コードの読みどころ

1. `seseragi.toml`: パッケージ名、実行の入口（`entry = "main"`）、`seseragi test` 用の設定を宣言します。
2. `pricing/shipping.ssrg`: 外部から使う型と計算関数だけに `pub` を付け、地域別基本料金などの細部は非公開にしています。表示用の文字列はここには持ちません。
3. `pricing/format.ssrg`: 計算結果を表示用の文に整える責務だけを分けています。
4. `main.ssrg`: サンプルデータを用意して計算と表示をつなぐだけに保っています。
5. `tests/shipping.ssrg`: 本体のコードから分離した単体テストです。手元の環境で `seseragi test examples/samples/project-greeting` を実行して確認できます。

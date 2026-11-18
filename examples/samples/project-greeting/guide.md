このサンプルを選ぶ理由: 計算ロジック、確認用のチェック、エントリポイントを別ファイルに分けたパッケージの構成を見たいときに選びます。

送料計算を行う `pricing/shipping.ssrg`、仕様通りの結果かを `==` で確かめる `pricing/checks.ssrg`、結果を表示する `main.ssrg` に役割を分け、Explorer から切り替えて編集できます。手元で `seseragi test` を実行すると `tests/shipping.ssrg` の単体テストが走ります。

## この画面で試すこと

- Try next の「珈琲豆を 5000 円にして送料無料を見る」を押し、国内配送が 5000 円以上で送料無料に切り替わるのを見る
- 「配送見積もりに箱物を 1 件足す」を押し、2kg 超えの重量追加料金（350 円）が加算されるのを見る
- `pricing/shipping.ssrg` の `baseShipping` を `main.ssrg` から直接呼ぼうとして、`pub` のない関数がモジュール外に漏れないことを確かめる

## コードの読みどころ

1. `seseragi.toml`: パッケージ名、実行の入口（`entry = "main"`）、`seseragi test` 用の設定を宣言します。
2. `pricing/shipping.ssrg`: 外部から使う型と計算関数だけに `pub` を付け、地域別基本料金などの細部は非公開にしています。
3. `pricing/checks.ssrg`: 計算モジュールを `import` し、代表ケースを `==` で比較してパス件数を表示します。
4. `main.ssrg`: サンプルデータを用意して計算と検証の結果をつなぐだけに保っています。

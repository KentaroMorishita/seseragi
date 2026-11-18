このサンプルを選ぶ理由: JSON 文字列を型に安全に読み込み、壊れた入力の場所や理由をそのままプログラムで扱いたいときに選びます。

設定用の構造体 `ServiceConfig`（入れ子の `Endpoint` を含む）に `JsonEncode` と `JsonDecode` を自動導出し、正常な JSON、項目不足、型違い、構文エラーの 4 パターンを読み込みます。

## この画面で試すこと

- Try next の「足りない tls 項目を足す」を押し、2 番目の入力が `OK` になって再出力されるのを見る
- 「配列に数値を混ぜて型エラーを見る」を押し、`regions.[1]` の位置付きで型エラーが出るのを見る
- 正常な JSON に未知の項目 `,"extra":1` を足して、決められた構造以外の項目も安全に弾かれることを確かめる

## コードの読みどころ

1. `deriving Eq, Show, JsonEncode, JsonDecode`: 手書きの変換コードを書かずに、型定義から双方向の変換を自動で用意します。
2. `describeError`: `JsonReadError` が構文の誤り（`JsonSyntaxFailure`）と型の不一致（`JsonDecodeFailure`）を型レベルで分けます。
3. `formatPath` と `formatKind`: `endpoint.port` や `regions.[1]` のように、どこで何の型が足りなかったかを取り出せます。
4. `checkConfig`: 変換した `ServiceConfig` を `json.encodeString` で再び JSON 文字列へ戻し、値と文字列の往復を確かめます。

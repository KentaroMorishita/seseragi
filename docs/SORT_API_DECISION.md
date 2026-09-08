# Collection sorting API decision (#369)

O01 では任意 comparator を受け取る `sortWith` を追加しない。これは次期監査への先送りではなく、現在の標準 API に採用しないという判断。

`Ord<A>` は型の canonical order を所有する。画面表示順や一時的な優先度は `sortBy` で順位 key へ射影できる。例えば `arrays.sortBy (\value -> if value == "urgent" then 0 else 1) values` は urgent を先にし、同順位の要素順を stable sort で保持する。List も同じ契約である。複数優先度は小さい順位値へ明示的に写像できる。

任意 comparator は key に表せない pair-dependent ordering も表現できるため、`sortBy` と完全に同じ機能ではない。しかし今回の Issue にはその機能を必要とする具体的な application case がなく、比較関数の整合性違反、呼出し回数・順序、Array/List 間の挙動を新たに標準契約へ加える根拠が足りない。既存の source 順一回の key 評価と stable sort を維持する。

新しい comparator API、local/orphan `Ord` instance、JS comparator number を source へ露出する経路は追加しない。採用判断を変える場合は、具体的な利用例を根拠に別途規範を変更する。#369 の完了には本決定と通常の Array/List `sortBy` 経路の確認を用いる。

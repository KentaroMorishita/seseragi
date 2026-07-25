`alias`は既存の型へ読みやすい名前を付ける透明な仕組みです。`UserId`は`Int`と、`Pair<Int>`は対応するrecordと同じ型として扱われ、runtimeの包みは増えません。

`Task<Unit>`も標準の透明aliasで、正規型は`Effect<{}, Never, Unit>`です。`signals.update`のように環境も回復可能failureも不要なEffectを短く表します。

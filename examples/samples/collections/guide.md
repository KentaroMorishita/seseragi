Array、List、Rangeをconcrete型ごとの別APIへ分けず、genericな集約で処理するrecipeです。

- `product`は`one ()`からsource順に乗算します。
- `any`と`all`は結果が決まった時点で走査を止めます。
- `join`と`combine`はStringを読みやすく組み立てます。
- `std/array`と`std/list`は、長さや先頭要素を型安全に読み取ります。
- 見つからないかもしれない`find`は`Maybe`で受け取り、`match`で扱います。
- `filterMap`は変換と絞り込みを一度に行い、`Nothing`だけを除きます。

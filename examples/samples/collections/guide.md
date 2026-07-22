Array、List、Rangeをconcrete型ごとの別APIへ分けず、genericな集約で処理するrecipeです。

- `product`は`one ()`からsource順に乗算します。
- `any`と`all`は結果が決まった時点で走査を止めます。
- `join`と`combine`はStringを読みやすく組み立てます。

一つの`completedSales`を`map`による表示行と、genericな`sum`による合計の両方へ使うrecipeです。`join`は行内の区切りと改行を組み立て、0円のキャンセル行は`filter`で一度だけ除外します。

`filter`と`map`は`std/array`の型を保つ変換です。一方、`sum`と`join`は`Reducible`を通るgeneric APIなので、同じ考え方をListやRangeにも使えます。

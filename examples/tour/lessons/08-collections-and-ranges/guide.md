`[1, 2, 3]`はArrayです。opening bracketの直前にbacktickを置くとpersistent Listになり、`1..=4`は終端を含むRangeになります。いずれも有限なcollectionとして処理できます。

`filter`はBoolを返すpredicateに合う要素だけを残し、`map`は各要素へ関数を適用します。`%`は剰余、`==`は等値比較なので、`value % 2 == 0`は偶数ならTrueです。`[value | value <- range]`はRangeを順に読み、Arrayを組み立てるcomprehensionです。

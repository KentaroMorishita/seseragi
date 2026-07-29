ADTは取り得る形をconstructorで閉じた型です。`Delivery`は値を持たない`Preparing`か、`String`を一つ持つ`Shipped`のどちらかです。

`match`は値のconstructorを調べ、対応するarmの右側を返します。`Shipped city`のpattern bindingはpayloadを`city`という名前で安全に取り出します。

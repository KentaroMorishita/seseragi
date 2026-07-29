`Maybe<A>`は値がある`Just A`と、値がない`Nothing`を型で区別します。値がない可能性を特別なStringや数値へ隠しません。

`Either<E, A>`は失敗値の`Left E`と成功値の`Right A`を区別します。どちらも前lessonのADTと同じようにmatchできるため、payloadを取り出す前にすべてのconstructorを扱えます。

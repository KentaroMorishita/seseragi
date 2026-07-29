`keep<A>`の`A`は呼び出しごとに具体型が決まるtype parameterです。関数本体は型固有の操作を仮定せず、どの`A`でも同じ形で値を返します。

`trait Label<A>`は型が提供する振る舞いの契約で、`instance Label<Score>`がScore用の実装を与えます。`impl Score`の`operator +`も標準Add契約へつながり、呼び出し側では選ばれた型のevidenceを通して使われます。

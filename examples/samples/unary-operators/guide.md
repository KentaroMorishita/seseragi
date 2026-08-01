unary `-`と`!`を、注釈なしのbinding、直接argument、行頭から始まるpipelineで使います。結果はgenericな関数適用を通っても、Showを選ぶtemplate / `printValue`とdeveloper向けのDebugへ同じ型のまま渡ります。`-0.0`はFloatの符号を保持し、どのsurfaceでも`-0.0`と表示されます。

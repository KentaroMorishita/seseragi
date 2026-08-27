このsampleは、named APIだけでなく標準演算子が選択したinstance dictionaryを実際に実行する様子を横断して確認します。

`<$>`は`Functor.map`、`<*>`は`Applicative.apply`、`>>=`は`Monad.flatMap`へdesugarされます。同じ演算子でも、値の外側の型がMaybe、Either、Array、List、Effect、Task、Stream、Signalのどれかによって対応するdictionaryが選択されます。

SignalはFunctorとApplicativeまでを提供します。最後のSignal例は`*signal`で現在値をEffectとして読み、Stream例は`runCollect`でcold streamを実行してArrayへ集めます。Signalへ`>>=`を使うと、Monad instanceがないため診断になります。

出力の各行は`map` / `apply` / `flatMap`の順に、named methodの結果とoperatorの結果を`|`で並べています。Signalは`map` / `apply`だけを比較し、`>>=`は負例fixtureで確認します。

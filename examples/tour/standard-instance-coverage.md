# Standard instance / Tour coverage audit

この表は`docs/spec/09-standard-library.md`のFunctor / Applicative / Monad
instanceを、canonical Tourでの初出と意図的な対象外判断へ対応付けます。
compiler instanceの有無ではなく、通常codeを読むためにTour本編で何を具体型として
体験させるかを記録します。法則、evidence、instance設計は#260 Deep Diveの責務です。

| 型 | Functor | Applicative | Monad | 型自体 | Named operation | Operator | `do` | Tourでの判断 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `Maybe` | yes | yes | yes | `maybe-*` | `map` / `apply` / `flatMap` | `<$>` / `<*>` / `>>=` | 抽象化章で三表記を比較 | fallibility章で三operationを具体的に初出し、抽象化章でFunctor / Applicative / Monadへ回収する |
| `Either<E, _>` | yes | yes | yes | `either-*` | Right側の`map` | `<$>` | なし | `either-map-error`でRight変換とLeft保持を体験する。`apply` / `flatMap`の反復は必修Tourを増やさず、共通contractと最初のLeftで止まるsemanticsを抽象化章で明示する |
| `Array` | yes | yes | yes | `array-*` | `map` | `<$>` | なし | `collection-map`でListと並べてshape保持を確認する。Cartesian `apply` / `flatMap`は初学経路へ無理に追加せずDeep Dive / Reference候補とする |
| `List` | yes | yes | yes | `list-*` | `map` | `<$>` | なし | `collection-map`でArrayだけのcoverageを禁止する。persistent shapeを出力で比較し、Cartesian `apply` / `flatMap`はArrayと同じ判断にする |
| `NonEmptyList` | yes | yes | yes | なし | なし | なし | なし | 型自体が現行必修Tour外。non-empty保証とCartesian semanticsを扱う専用RecipeまたはDeep Diveを別Issueで設計し、本Issueでは捏造しない |
| `Effect` / `Task` | yes | yes | yes | `effect-*`（TaskはEffect aliasとして扱う） | `flatMap` | `>>=` | `10-effects-and-do` | Maybeで既習のbindをEffectのsuccess channelへ再適用し、doが別物ではないことを具体codeで確認する。operator中心のEffect章にはしない |
| `Signal` | yes | yes | no | `signals-*` | `signals.map` / `pure` / `apply` | `<$>` / `<*>` | なし | 既存Signal章を具体的な再適用先として維持する。`signals-monad-boundary`で`>>=`を提供しない契約をdiagnosticまで固定する |
| `Stream<R, E, _>` | yes | yes | yes | なし | なし | なし | なし | Streamは現行Tourの`excludedDesignSurfaces`。cold / sequential semanticsを実装済みsurfaceとして扱える段階でRecipe / Deep Diveへ切り出す |
| `Validation<E, _>` | yes | yes | no | なし | なし | なし | なし | 型自体が現行必修Tour外。error accumulationをEitherの短絡semanticsへ混ぜず、Applicative専用教材を別Issueで判断する。Monadは仕様上提供しない |

## 学習順の固定

1. `collection-map`でArray / Listの`map`と`<$>`を初出する。
2. `maybe-map`、`maybe-combine`、`maybe-short-circuit`でMaybeのFunctor、
   Applicative、Monad表記を一つずつ体験する。
3. `either-map-error`でEitherのRight変換とLeft保持を確認する。
4. `10-effects-and-do`でEffectの`flatMap` / `>>=` / `do`を対応付ける。
5. `abstraction-*`は上記lessonをprerequisiteにし、`F<_>`、instance選択、
   型ごとのsemantics、Monadを持たない境界を回収する。
6. Signal章では同じ表記を時間変化する具体型へ再適用し、no-Monad境界を確認する。

`requiredTopics`は型別のoperator surfaceを別々に持ち、Arrayだけ、Maybeだけ、または
抽象化章に一度operatorが現れるだけではcoverageを満たせません。

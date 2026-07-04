# Seseragi仕様索引

この索引は、言語機能の所在と章をまたぐ不変条件をまとめます。各章は実装状況ではなく、
Seseragiが保証する意味を記述します。

## 読む順序

1. [言語の定義](./00-language.md)
2. [字句・構文・演算子](./01-syntax.md)
3. [型システム](./02-types.md)
4. [データ・式・パターン](./03-data-and-expressions.md)
5. [型クラスとdo notation](./04-type-classes.md)
6. [失敗・Effect・Signal](./05-effects.md)
7. [Seseragi module](./06-modules-and-interop.md)
8. [TypeScript interop](./07-typescript-interop.md)
9. [`.d.ts` binding生成](./08-dts-conversion.md)
10. [標準ライブラリ契約](./09-standard-library.md)
11. [標準ライブラリsurface](./10-library-surface.md)
12. [Parser・formatter・language server契約](./11-tooling.md)
13. [Appendix A: 文法要約](./grammar.md)

## feature map

### 構文

- 空白によるcurried function application
- `if` / `match` / block expression
- lambda、pipeline、低優先順位application
- Maybe専用の短絡fallback `??`
- 固定operator、struct operator overload、userland custom infix operator
- genericなdo notation
- Signal snapshot read `*signal`
- Signal専用 `:=`

### 型

- rank-1 let-polymorphism
- 明示型parameterとcall-site inference
- user-defined generic ADT、struct、alias、method
- kind checkingとhigher-kinded parameter `M<_>`
- nominal ADT/struct、structural immutable record
- trait constraint、coherent instance、operator functional dependency
- invariant generic型と、明記されたcapability coercion

### データと抽象化

- ADT、record、struct、tuple、Array、List、両collectionのcomprehensionとpattern
- 網羅的pattern match
- Eq、Ord、Show、Debug、Hashの限定的なderiving
- Showで検査される純粋なtemplate interpolation
- Functor、Applicative、Monad、Semigroup、Monoid
- do notationとmonad transformer

### Effectと状態

- `Effect<R, E, A>`
- `Task<E, A> = Effect<{}, E, A>`
- typed error、environment service、resource、cancellation、parallelism
- Signal / MutableSignal、transaction、glitch-free propagation、subscription lifetime
- Ref、Stream、Fiberなどの標準runtime abstraction

### module

- 1 file = 1 module
- package identityとmanifest dependency
- private-by-default、`pub`、opaque型
- named / namespace import、re-export、custom operator import
- canonical path解決、循環禁止、deterministic初期化

### 外部連携

- `pure` / `task` foreign call mode
- nullable、Promise、throw、class、callbackの明示境界
- TypeScript向けstable wrapper ABI
- `.d.ts` subset変換とunsupported診断
- `any` や高度なTypeScript型を黙って通常型へ弱めない規則

### 標準ライブラリ

- Maybe、Either、Validation
- collection、text、number、JSON、time、random
- Effect、Stream、Signal、concurrency、resource
- Console、Logger、filesystem、process、HTTP
- testing、law testing、deterministic test service

### tooling

- compiler / formatter / language serverで共有するlossless CST
- operator header scan、module interface、fixity resolution
- incomplete sourceとunknown operatorのerror recovery
- dependency fixity変更のincremental invalidation

## cross-cutting invariants

1. 通常の値は不変で、外部状態を変える操作はEffectを返す。
2. Taskは独立型ではなく、environment不要なEffectのaliasである。
3. showは純粋な文字列化で、print/logはservice requirementを持つEffectである。
4. do notationは任意のMonadに使えるが、異なるMonadを暗黙liftしない。
5. standard operator overloadはtrait instance、custom operatorは名前付き多相関数である。
6. type aliasはinstance identityを作らず、nominal identityにはADTかstructを使う。
7. foreign値は境界型を経由し、TypeScriptの型をSeseragi内部型として扱わない。
8. `.d.ts` converterはbinding候補を生成し、Seseragi型システムをTypeScriptへ従属させない。
9. module importだけではEffectを実行せず、I/Oを発生させない。
10. 現行compilerの挙動や対応範囲は、言語仕様の意味を変更しない。
11. derivingとtemplate展開は仕様で閉じ、汎用マクロとして任意のcode生成を許さない。
12. compiler、formatter、language serverは同じtoken境界とoperator結合規則を使う。

## 明示的に採用しないもの

- mutable variable / mutable field
- `null` / `undefined` を通常値として扱うこと
- implicit numeric conversionとtruthiness
- non-exhaustive match
- exceptionによる通常のerror handling
- arbitrary union / intersection type
- overlapping / orphan trait instance
- implicit monad transformer lift
- `perform` / `handle` algebraic effect
- user-definedな汎用構文マクロと手続きマクロ
- user-defined prefix / postfix operator
- Listのhead `^`、tail `>>`、連結記法
- reverse pipeline `~`
- 三項演算子
- 空白だけによるmethod call
- 特別なmonoid宣言とfold operator `>>>`
- SignalのMonad instance
- TypeScript `any` の無警告受け入れ

## 仕様変更の条件

機能を追加・変更するときは、最低でも次を同時に更新します。

- source syntaxまたは公開API
- 型付けと推論規則
- 評価順序とfailure条件
- module visibilityとinterop境界
- 標準ライブラリとの関係
- Appendixの文法

例だけを追加して意味規則を後回しにしてはなりません。

# Seseragi仕様カバレッジ

この文書は仕様整理用の非規範チェックリストです。言語の意味は `docs/spec/` を正とします。
実装計画や特定backendの移行計画は扱いません。

## 定義済み

### 言語の核

- 式中心、strict evaluation、左から右の評価順序
- 不変bindingと不変data
- curry、部分適用、rank-1 let-polymorphism
- generic function / ADT / struct / alias / impl
- Eq / Ord / Show / Debug / Hashの限定的なderiving
- Showで型検査されるtemplate interpolation
- kind、arity、型構築子parameter
- nominal型とstructural record
- ADT、網羅的match、pattern guard

### 抽象化

- coherent type classとorphan rule
- Functor / Applicative / Monad
- Semigroup / Monoid
- standard operator overloadとfunctional dependency
- userland custom infix operator
- generic do notation
- monad transformerの標準形

### Effectと状態

- Effectのenvironment / error / success channel
- Task alias
- 順次・並列、resource、cancellation、defect
- Signal / MutableSignal
- transaction、glitch-free更新、subscription lifetime
- Console / LoggerとShowの分離

### moduleと外部境界

- Seseragi module identity、visibility、import、re-export
- package dependency、循環禁止、初期化順序、entry point
- TypeScript foreign binding、call mode、class / callback / Promise
- TypeScript向け公開ABI
- `.d.ts` subset変換、unsupported診断、更新差分

### 標準ライブラリ

- preludeの境界
- collection / text / number / JSON / time / random
- Effect / Stream / Signal / concurrency
- Console / Logger / filesystem / process / HTTP
- test runtimeとlaw test

## 追加定義が必要

以下は必要性を確認済みですが、正本で完全な契約まで定義していません。

### boilerplate削減

- nominal wrapperを簡潔に書くnewtype相当の構文
- record / struct decoderとencoderの導出

### packageとtooling contract

- package manifestの具体的schema
- package export mapとversion compatibility
- source root、test root、generated binding rootの標準layout
- formatterのcanonical output
- document commentとAPI document生成規則

### runtime contract

- scheduler fairnessとFiber supervision
- Queue / Ref / Deferred / Semaphoreの完全な型とcancellation semantics
- Streamのbackpressure、merge ordering、buffer overflow policy
- SignalとStream間変換のloss policy
- process signalとgraceful shutdown

### 標準data

- Bytesのrepresentationとslice ownership
- Decimalのprecision / rounding context
- Map / Setのhash seedとserialization contract
- Regex flavorとUnicode version
- timezone databaseのversioning

### interop詳細

- TypeScript generic関数を公開する `.d.ts` ABI
- callback resourceの生成設定
- discriminated union converterのnaming rule
- declaration mergeとnamespace exportの生成名衝突
- source mapとstack traceのcross-language表示

### 診断とlanguage service

- diagnostic codeとseverity
- type inference traceの表示
- non-exhaustive matchのfix suggestion
- custom operator / generic angle bracket ambiguityのformatter rule
- incomplete sourceを扱うrecovery grammar

## 意図的に採用しない

- mutable variable / mutable field
- nullとundefinedの通常値化
- implicit numeric conversionとtruthiness
- arbitrary union / intersection type
- exceptionによる通常error handling
- overlapping / orphan instance
- implicit transformer lift
- algebraic `perform` / `handle`
- user-definedな汎用構文マクロと手続きマクロ
- user-defined prefix / postfix operator
- TypeScript checkerへの型意味論の委譲

## 次に詰める順序

1. newtypeを決め、日常的なdata modelingのboilerplateを確定する。
2. package manifestと標準project layoutを決め、module仕様を閉じる。
3. standard collection / Effect / Stream / Signalの公開signatureをmodule単位で固定する。
4. runtime concurrency contractを固定する。
5. diagnostic、formatter、document commentをtooling contractとして固定する。
6. TypeScript binding generatorのnamingと設定schemaを固定する。

この順序は実装順ではなく、仕様の依存順です。

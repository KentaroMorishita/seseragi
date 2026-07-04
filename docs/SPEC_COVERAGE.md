# Seseragi仕様カバレッジ

この文書は仕様整理用の非規範チェックリストです。言語の意味は `docs/spec/` を正とします。
実装計画や特定backendの移行計画は扱いません。

## 現行機能を扱う基準

現行compilerの挙動は設計資料であり、互換性の正本ではありません。独自記法を見つけた場合は、
次のいずれかに分類してから正本へ反映します。

- 維持: Seseragiの読み味を作り、型と評価規則も一貫している。
- 再設計: 表面構文は有用だが、現行の型・失敗・副作用の意味に問題がある。
- 廃止: 通常関数より理解しにくい、別概念を同じ記号へ混在させる、または安全性を隠す。
- 保留: 言語全体との関係を決めるまで意味を確定できない。

採用する機能は、少なくとも構文、型、評価順序、展開先、失敗条件、module境界を定義します。
現行testが通ることやTypeScriptへ生成できることだけを、仕様採用の根拠にはしません。

## 定義済み

### 言語の核

- 式中心、strict evaluation、左から右の評価順序
- 不変bindingと不変data
- curry、部分適用、rank-1 let-polymorphism
- generic function / ADT / struct / alias / impl
- Eq / Ord / Show / Debug / Hashの限定的なderiving
- Showで型検査されるtemplate interpolation
- Array / Listのliteral、pattern、comprehension、明示変換
- Maybe専用の短絡fallback `??`
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
- Signal snapshot read `*signal` と更新 `:=`
- transaction、glitch-free更新、subscription lifetime
- Console / LoggerとShowの分離

### moduleと外部境界

- Seseragi module identity、visibility、import、re-export
- package dependency、循環禁止、初期化順序、entry point
- TypeScript foreign binding、call mode、class / callback / Promise
- TypeScript向け公開ABI
- `.d.ts` subset変換、unsupported診断、更新差分

### parserとtooling

- raw operator scanとheader scanの分離
- module interfaceによるcustom operator fixity解決
- lossless CSTとflat operator chain
- incomplete source、unknown operator、missing tokenのrecovery contract
- compiler / formatter / language server間の構文一致
- fixity変更によるdependent documentのinvalidation

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
- Listのhead `^`、tail `>>` と連結記法
- reverse pipeline `~`
- 三項演算子
- 空白だけによるmethod call
- `monoid` 特別宣言とfold operator `>>>`
- SignalのMonad instance。動的dependency切替は名前付き `switchMap` を使う
- TypeScript checkerへの型意味論の委譲

## 次に詰める順序

1. newtypeを決め、日常的なdata modelingのboilerplateを確定する。
2. package manifestと標準project layoutを決め、module仕様を閉じる。
3. standard collection / Effect / Stream / Signalの公開signatureをmodule単位で固定する。
4. runtime concurrency contractを固定する。
5. diagnostic、formatter、document commentをtooling contractとして固定する。
6. TypeScript binding generatorのnamingと設定schemaを固定する。

この順序は実装順ではなく、仕様の依存順です。

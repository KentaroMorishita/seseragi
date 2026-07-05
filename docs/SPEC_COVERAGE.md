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
- generic function / ADT / struct / newtype / alias / impl
- Eq / Ord / Show / Debug / Hashの限定的なderiving
- Showで型検査されるtemplate interpolation
- Array / Listのliteral、pattern、comprehension、明示変換
- Maybe専用の短絡fallback `??`
- kind、arity、型構築子parameter
- nominal型とstructural record
- entry pointでfield集合を確定するclosed structural record
- ADT、網羅的match、pattern guard
- 単一値のnominal wrapperを表すnewtypeとopaque境界

### 抽象化

- coherent type classとorphan rule
- Functor / Applicative / Monad
- collection / Effect / Stream instance semanticsとValidation error accumulation
- Semigroup / Monoid
- standard operator overloadとfunctional dependency
- userland custom infix operator
- generic do notation
- monad transformerの標準形とmodule-qualifiedなrun / lift API

### Effectと状態

- Effectのenvironment / error / success channel
- Effect / Streamのrequirement wideningとNever failure widening
- 正規Effect型へ展開される `effect fn` とeffectful `for`
- Task alias
- 順次・並列、resource、cancellation、defect
- Signal / MutableSignal
- Signal snapshot read `*signal` と更新 `:=`
- transaction、glitch-free更新、subscription lifetime
- SignalChangeによるmulti-signal transactionとswitchMap lifetime
- cooperative schedulerのweak fairnessとstructured Fiber supervision
- Queue / Ref / Deferred / Semaphoreの型・順序・cancellation semantics
- Effect timeout、Schedule retry / repeat、resource scopeとfinalizer semantics
- process signal mode、root cancellation、grace period、exit status
- Streamのdemand、merge ordering、buffer overflow、resource lifetime
- Signal / Stream変換の初期値、backpressure、loss policy
- Console / LoggerとShowの分離
- immutable Bytes、opaque Byte、slice sharing / copy、UTF-8変換

### moduleと外部境界

- Seseragi module identity、visibility、import、re-export
- namespace aliasによる値・型・constructor・traitのqualified参照
- package dependency、循環禁止、初期化順序、entry point
- `seseragi.toml`、export map、標準layout、generated root、lockfile
- TypeScript foreign binding、call mode、class / callback / Promise
- TypeScript向け公開ABI
- `.d.ts` subset変換、unsupported診断、更新差分
- unconstrained generic TypeScript ABIと明示trait dictionary mapping
- callback lifetime設定、stable generated naming、declaration merge / namespace変換
- source map chainとcross-language machine-readable stack frame

### parserとtooling

- raw operator scanとheader scanの分離
- reserved wordとAppendix grammarの機械的整合
- record/struct value・pattern・shorthand・spreadの完全なgrammar
- module interfaceによるcustom operator fixity解決
- lossless CSTとflat operator chain
- incomplete source、unknown operator、missing tokenのrecovery contract
- compiler / formatter / language server間の構文一致
- fixity変更によるdependent documentのinvalidation
- lexical / semantic syntax highlightのtoken契約
- playgroundとCLIのfrontend・example source共有
- example単位のcompiler / formatter / LSP / highlight / playground conformance
- `.stdout` snapshotを含むbyte単位のexample出力検証
- stable diagnostic code / span / fix schemaとtype inference explanation
- canonical idempotent formatterとrange-format recovery
- `///` / `//!` document comment、deterministic API docs、doctest
- non-exhaustive matchのconstructor-aware code action

### 標準ライブラリ

- preludeの境界
- collection / text / number / JSON / time / random
- Decimalのcanonical value、exact arithmetic、明示的なprecision / rounding context
- Map / Setのprocess-local hash seed、ordered / canonical serialization contract
- linear-time Regex subset、UTF-8 span、toolchain共通のUnicode data version
- timezoneの明示的なlocal resolution、IANA database version locking
- portable Path、filesystem error、stream / atomic / temporary resource ownership
- child processのcold event Stream、capture limit、termination / reap ownership
- HTTPのcold exchange Stream、body backpressure、connection ownership
- persistent IteratorとIterable/Reducibleの要素型dependency
- Array / List / NonEmptyListの公開signature、境界値、反復順、計算量
- Effect / Stream / Signal / concurrency
- Console / Logger / filesystem / process / HTTP
- test runtimeとlaw test

### executable design example

- `examples/spec/lessons/` を難易度順の学習教材として管理
- `examples/spec/fixtures/` をpositive / diagnostic / multi-file conformance targetとして管理
- `examples/spec/COVERAGE.md` で全仕様機能をlessonまたはfixtureへ対応づける
- 一般的なprogramからstdlib不足とhost requirement矛盾を発見する
- playground sampleへsourceを複製せず生成・直接読込する

## 追加定義が必要

以下は必要性を確認済みですが、正本で完全な契約まで定義していません。

### boilerplate削減

- record / struct decoderとencoderの導出

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

1. record / struct decoderとencoderの導出規則を固定する。
2. 確定した契約へpositive / diagnostic / project fixtureを追加する。

この順序は実装順ではなく、仕様の依存順です。

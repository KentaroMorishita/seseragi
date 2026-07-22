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
- optional record fieldのpresence、width subtyping、query pattern、JSON / TypeScript境界
- entry pointでfield集合を確定するclosed structural record
- Effect / Stream第一型引数だけのrestricted requirement merge
- ADT、網羅的match、pattern guard
- 単一値のnominal wrapperを表すnewtypeとopaque境界

### 抽象化

- coherent type classとorphan rule
- inherent `impl`とtrait `instance`の構文上の分離
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
- compact inferred `effect fn` のbody由来contract推論とartifact固定
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
- canonical hex / Base64 / Base64url codecとtyped decode offset
- Unicode normalization / property APIとextended grapheme cluster境界

### moduleと外部境界

- Seseragi module identity、visibility、import、re-export
- namespace aliasによる値・型・constructor・traitのqualified参照
- package dependency、循環禁止、初期化順序、entry point
- `seseragi.toml`、export map、標準layout、generated root、lockfile
- canonical TOML lockfile schema、content digest、dependency edge、stale detection
- TypeScript foreign binding、call mode、class / callback / Promise
- TypeScript向け公開ABI
- `.d.ts` subset変換、unsupported診断、更新差分
- unconstrained generic TypeScript ABIと明示trait dictionary mapping
- callback lifetime設定、stable generated naming、declaration merge / runtime namespace変換
- resolved host module単位のpure-load / task-load、single-flight、failure memoization
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
- closed deprecation metadata、LSP tag、API docs / generated binding伝播
- stable CLI option schemaとbuild-time target capability query
- non-exhaustive matchのconstructor-aware code action

### 標準ライブラリ

- preludeの境界
- collection / text / number / JSON / time / random
- Int / Floatのcanonical parse / format、丸め、checked / saturating / wrapping arithmetic
- Decimalのcanonical value、exact arithmetic、明示的なprecision / rounding context
- Map / Setのprocess-local hash seed、ordered / canonical serialization contract
- linear-time Regex subset、UTF-8 span、toolchain共通のUnicode data version
- timezoneの明示的なlocal resolution、IANA database version locking
- portable Path、filesystem error、stream / atomic / temporary resource ownership
- process current directoryとfilesystem相対Pathの共通portable Path基準
- child processのcold event Stream、capture limit、termination / reap ownership
- HTTPのcold exchange Stream、body backpressure、connection ownership
- JsonEncode / JsonDecodeの限定deriving、strict field / tagged ADT wire contract
- persistent IteratorとIterable/Reducibleの要素型dependency
- Array / List / NonEmptyListの公開signature、境界値、反復順、計算量
- Effect / Stream / Signal / concurrency
- Console / Logger / filesystem / process / HTTP
- test runtimeとlaw test

### executable design example

- `examples/spec/lessons/` を完成仕様のdesign curriculumとして管理
- `examples/samples/` を現行compilerで実行可能な学習・発見catalogとして管理
- `examples/spec/fixtures/` をpositive / diagnostic / multi-file conformance targetとして管理
- `examples/spec/COVERAGE.md` で全仕様機能をlessonまたはfixtureへ対応づける
- 一般的なprogramからstdlib不足とhost requirement矛盾を発見する
- Playgroundとnative CLIが`examples/samples/`の同じsourceを自動発見して実行する
- token / lossless CST / diagnostic / module interfaceのversioned artifact contract

### Web UI

- optional props recordとIntoChildrenによるpure Html tree
- Msgを生成するevent snapshotとbounded sequential dispatch
- Signal transaction単位のkeyed DOM reconciliation
- DOM mount / listener / subscriptionのresource ownership
- deterministic SSR、strict / replace hydration、raw HTML禁止

### 性能モデル

- 意味論上の性能保証、消去可能性、quality of implementationの分離
- type alias、型引数、newtype、surface sugarのerasure contract
- curry direct call、trait specialization、collection / Stream fusionの許可条件
- direct self tail callとEffect / Stream chainのstack safety
- Effect、Signal、DOM、foreign境界で残るruntime cost
- development / release profileの意味的一致
- semantic differential、IR shape、benchmarkの三層検証
- benchmark value discovery、measurement metadata、portable baseline / regression gate

## 今回の基礎監査で残した未定義項目

一般application、product運用、toolingの基礎surfaceとして洗い出した項目は正本へ移し終えました。
今後新しい未定義点が見つかった場合は、exampleや現行compilerから意味を推測せず、ここへ再登録してから
設計します。現在登録中の未定義項目はありません。standard API監査で再登録したBigInt surfaceは、
`std/big-int`のAPI、算術、failure、変換、cost contractとして正本へ移しました。

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
- `return` / `break` / `continue`による非局所control transfer。短絡は目的別combinatorと通常値で表す
- Listのhead `^`、tail `>>` と連結記法
- reverse pipeline `~`
- 三項演算子
- 空白だけによるmethod call
- `monoid` 特別宣言とfold operator `>>>`
- SignalのMonad instance。動的dependency切替は名前付き `switchMap` を使う
- TypeScript checkerへの型意味論の委譲

## 次に詰める順序

1. 章をまたぐ意味論と性能境界を横断監査する。
2. 確定した契約へpositive / diagnostic / project / IR shape fixtureを追加する。
3. fixture runnerをformatter、LSP、highlight、playgroundと共有する。

この順序は実装順ではなく、仕様の依存順です。

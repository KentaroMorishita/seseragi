# Seseragi 学習コース

このdirectoryは、人がSeseragiを順番に学ぶための実行可能なlessonです。網羅性だけを目的とする
小さなcompiler fixtureは `../fixtures/` に分離します。

各lessonは次を守ります。

- 冒頭に学習目標と前提lessonを書く。
- 新しい概念の直前に、目的と意味を日本語commentで説明する。
- 新しく導入する中心概念を1〜2個に絞る。
- 既出構文は説明なしで再利用してよいが、未習概念を主題より先に要求しない。
- 同じ処理を複数の抽象で書く場合、意味の違いまたは等価性をcommentで説明する。
- 実行可能で、exact stdoutを持つ。

## 順序

| Lesson | 学ぶこと                     | 主な記法                                  |
| -----: | ---------------------------- | ----------------------------------------- |
|     01 | 最小programと副作用の型      | `effect fn`, `with`, `fails`              |
|     02 | 値・型・curried function     | `let`, `fn`, 空白適用, `$`                |
|     03 | 条件分岐とpattern matching   | `if`, `match`, tuple pattern              |
|     04 | Arrayとdata pipeline         | lambda, <code>&#124;&gt;</code>, `map`    |
|     05 | 基礎構文の統合               | range, effectful `for`, FizzBuzz          |
|     06 | collectionの畳み込み         | `reduce`, Map, nullish fallback           |
|     07 | domain data                  | `struct`, ADT, `newtype`, `deriving`      |
|     08 | 型付き失敗を含むpipeline     | `Either`, nested `map`                    |
|     09 | 独立した計算の合成           | `pure`, `<*>`, `Validation`               |
|     10 | 依存する計算とdoの位置づけ   | `>>=`, `map`, `do`                        |
|     11 | parametric polymorphism      | generic struct / method / alias           |
|     12 | 宣言的なcollection構築と分解 | comprehension, nested pattern             |
|     13 | user-defined abstraction     | trait, constraint, custom infix operator  |
|     14 | 意味を持つ集約               | `Semigroup`, `Monoid`, nominal wrapper    |
|     15 | reactive state               | `Signal`, `*`, transaction                |
|     16 | structured concurrency       | Fiber, Deferred, `join`                   |
|     17 | asynchronous sequence        | Stream, backpressure                      |
|     18 | resource lifetime            | scope, acquire/release                    |
|     19 | recursive generic data       | generic ADT, `rec`, record spread         |
|     20 | monad transformer            | `MaybeT`, qualified `lift` / `run`        |
|     21 | immutable binary data        | `Byte`, `Bytes`, slice, UTF-8 decode      |
|     22 | exact decimal arithmetic     | `Decimal`, explicit rounding              |
|     23 | safe text pattern matching   | Regex compile, typed failure, replacement |
|     24 | timezone and local ambiguity | IANA zone, DST gap / overlap              |
|     25 | safe filesystem access       | Path, temporary resource, typed I/O error |
|     26 | derived JSON codecs          | `JsonEncode`, `JsonDecode`, strict fields |
|     27 | pure HTML components         | props record, children, event Msg, SSR    |

ApplicativeをMonad/doより先に置きます。前の結果へ依存しない処理を最初からbind列へしないためです。
`do` はEffect専用構文ではなく、Lesson 10で `>>=` と対応づけてからadvanced lessonで使います。

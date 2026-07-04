# Learning path

このdirectoryは、人がSeseragiを順番に学ぶための実行可能なlessonです。網羅性だけを目的とする
小さなcompiler fixtureは `../fixtures/` に分離します。

各lessonは次を守ります。

- 冒頭に学習目標と前提lessonを書く。
- 新しく導入する中心概念を1〜2個に絞る。
- 既出構文は説明なしで再利用してよいが、未習概念を主題より先に要求しない。
- 同じ処理を複数の抽象で書く場合、意味の違いまたは等価性をcommentで説明する。
- 実行可能で、exact stdoutを持つ。

## 順序

| Lesson | 学ぶこと                                     | 主な記法                                 |
| -----: | -------------------------------------------- | ---------------------------------------- |
|     01 | expression、match、range、最小Effect program | `match`, `for`, `$`                      |
|     02 | data pipelineとgeneric collection操作        | <code>&#124;&gt;</code>, `map`, `reduce` |
|     03 | domain typeと型安全なoperator                | `newtype`, `deriving`, `impl`            |
|     04 | Array変換とtyped failure                     | lambda, `Either`, nested `map`           |
|     05 | 独立validationの合成                         | `pure`, `<*>`, `Validation`              |
|     06 | 依存計算とdoの位置づけ                       | `>>=`, `map`, `do`                       |
|     07 | reactive state                               | `Signal`, `*`, transaction               |
|     08 | structured concurrency                       | Fiber, Deferred, `join`                  |
|     09 | asynchronous sequence                        | Stream, backpressure                     |
|     10 | resource lifetime                            | scope, acquire/release                   |

ApplicativeをMonad/doより先に置きます。前の結果へ依存しない処理を最初からbind列へしないためです。
`do` はEffect専用構文ではなく、Lesson 06で `>>=` と対応づけてからadvanced lessonで使います。

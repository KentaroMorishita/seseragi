# Playground samples

新Rust compilerで現在実行できる言語能力を、入力を変えて試せるsingle-file programとしてまとめます。
小さいcompiler fixtureとは分け、Playgroundへ載せるsampleはdomain logicとEffect entryを両方持たせます。

| Sample | Stdin例 | 主な言語能力 |
| --- | --- | --- |
| `01-mini-adventure.ssrg` | `forest` / `rope` | 複数ADT、tuple pattern、typed failure |
| `02-shipping-advisor.ssrg` | `member` / `express` | domain分類、exhaustive match、pure function |
| `03-seseragi-quiz.ssrg` | `rust` | String pattern、ADT、Stdin / Console Effect |

各sourceは`apps/playground/tests/playground.integration.test.ts`からshared WASM driverでcompileし、generated
TypeScriptとbrowser runtimeを通した実行結果まで固定します。

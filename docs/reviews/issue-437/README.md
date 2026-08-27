# Issue #437 型クラス標準演算子の実行 parity

`examples/spec/fixtures/projects/typeclass-operator-parity/src/main.ssrg` を正本に、
標準演算子を parser から runtime まで同じ dictionary 選択で実行できることを確認する。
`<$>` は `Functor.map`、`<*>` は `Applicative.apply`、`>>=` は `Monad.flatMap` に対応し、
named method だけに置き換えた検証は採用しない。

## Coverage matrix

| value type | named method parity | `<$>` | `<*>` | `>>=` | selected dictionaries | execution evidence |
| --- | --- | --- | --- | --- | --- | --- |
| `Maybe<Int>` | `map` / `apply` / `flatMap` | yes | yes | yes | `std/maybe::{Functor,Applicative,Monad}` | CLI + WASM |
| `Either<String,Int>` | `map` / `apply` / `flatMap` | yes | yes | yes | `std/either::{Functor,Applicative,Monad}` | CLI + WASM |
| `Array<Int>` | `map` / `apply` / `flatMap` | yes | yes | yes | `std/array::{Functor,Applicative,Monad}` | CLI + WASM |
| `List<Int>` | `map` / `apply` / `flatMap` | yes | yes | yes | `std/list::{Functor,Applicative,Monad}` | CLI + WASM |
| `Effect<{},Never,Int>` | `map` / `apply` / `flatMap` | yes | yes | yes | `std/effect::{Functor,Applicative,Monad}` | CLI + WASM |
| `Task<Int>` (`Effect` alias) | `map` / `apply` / `flatMap` | yes | yes | yes | `std/effect::{Functor,Applicative,Monad}` | CLI + WASM |
| `Stream<{},Never,Int>` | `map` / `apply` / `flatMap` | yes | yes | yes | `std/stream::{Functor,Applicative,Monad}` | CLI + WASM + `runCollect` |
| `Signal<Int>` | `map` / `apply` | yes | yes | no | `std/signal::{Functor,Applicative}` | CLI + WASM; `>>=` negative diagnostic |

The same fixture invokes each named trait method and its operator counterpart, then is
compiled through Typed HIR, Core IR, and TypeScript lowering by
`crates/seseragi-driver/tests/compile_module.rs`. The test asserts the canonical trait-call
callees and every runtime dictionary import. `crates/seseragi-cli/tests/run.rs` checks that
the paired outputs agree, while `apps/playground/tests/playground.integration.test.ts`
executes the same source through the WASM project boundary.

The negative source in
`crates/seseragi-cli/tests/fixtures/typeclass-signal-monad-negative.ssrg` must report
`SES-T0201` (`no Monad instance matches`) for `Signal`; it must not silently fall back to a
named `flatMap` or produce a runtime-defect result.

## Runtime result

```text
Maybe: map Just 2|Just 2; apply Just 2|Just 2; flatMap Just 2|Just 2
Either: map Right 2|Right 2; apply Right 2|Right 2; flatMap Right 2|Right 2
Array: map 2,3|2,3; apply 2,3|2,3; flatMap 2,11,3,12|2,11,3,12
List: map 2,3|2,3; apply 2,3|2,3; flatMap 2,11,3,12|2,11,3,12
Effect: map 2|2; apply 2|2; flatMap 2|2
Task: map 2|2; apply 2|2; flatMap 2|2
Stream: map 2,3|2,3; apply 2,3|2,3; flatMap 2,3|2,3
Signal: map 2|2; apply 3|3
```

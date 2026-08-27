# Issue #433 Effect-aware tail recursion

## Root cause

`effect fn`のbodyはcoldなEffect thunkを返します。従来のTypeScript emitterは、
Effect thunkを実行した後の値へ現れるtail markerを、同期的なpure functionの
tail callと同じ形で扱っていました。そのため、`flatMap`のcallback内にある再帰
callの外側へ`while (true)`を置き、queue workerが最初のActionだけを消費して
runtime defectで停止していました。

## Decision

`TypedDecl::EffectFn`からCore / TypeScript IRへemitter専用の`is_effect` markerを
渡します。pure functionだけが現在の同期TCO loopの対象で、Effect functionは
`_ssrg_effect_flatMap(..., action => worker(...))`というruntime continuationを
保ちます。各Effectが解決した後に次のcallbackが呼ばれるため、effect-aware
loweringを導入するまでのhost stack safetyはEffect runtimeのcontinuationが担います。
markerはserialized IRへ出さず、既存artifact schemaを変更しません。

## Regression matrix

| boundary | evidence |
| --- | --- |
| lowering | `crates/seseragi-lowering/src/lib.rs`のeffect tail regressionが`is_effect`を確認し、生成bundleに`while (true)`がなく再帰callが残ることを確認 |
| runtime | `examples/spec/fixtures/projects/effect-tail-recursion`が`Add 1`, `Add 2`, `Add 3`, `Stop`を処理し、期待stdout `6`を生成 |
| CLI | `crates/seseragi-cli/tests/run.rs`がfixtureを実際に起動し、exit 0・stdout・stderrを検証 |
| pure boundary | 既存の`self-tail-loop`系テストはpure direct self tail recursionのloop loweringを引き続き検証 |

このfixtureは、Effectful tail recursionをpure TCOへ誤って昇格させず、queueの
複数Actionをruntime実行継続で最後まで処理することを固定します。

import assert from "node:assert/strict"
import {
  createEffectExecution,
  isEffectCancellation,
  run,
} from "@seseragi/runtime/effect"
import {
  choose,
  observe,
} from "./dist/packages/fixture/effect-match/0.0.0/domain.ts"

const calls: string[] = []
const environment = {
  console: {
    println(value: string) {
      calls.push(value)
      return { kind: "success" as const, value: undefined }
    },
  },
}
const effect = observe({ tag: "Just", value: 9 })
assert.deepEqual(calls, [], "constructing a matched Effect is cold")
assert.equal((await run(effect, environment)).kind, "success")
assert.equal((await run(effect, environment)).kind, "success")
assert.deepEqual(calls, ["9", "9"], "the same Effect can run twice")
assert.deepEqual(await run(choose({ tag: "Nothing" }), {}), {
  kind: "failure",
  error: "missing",
})
const execution = createEffectExecution()
await execution.cancel()
await assert.rejects(
  Promise.resolve().then(() => effect(environment, execution.context)),
  isEffectCancellation
)
assert.deepEqual(
  calls,
  ["9", "9"],
  "cancelled branches never reach the console"
)
await execution.close()
console.log("coldness, failure, cancellation: ok")

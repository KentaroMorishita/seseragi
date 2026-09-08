// Runtime-boundary companions to the compiler-generated quality.ssrg cases.
import { benchmark, blackBox, inputSize, suite } from "../../src/benchmark"
import { fromUint8Array, toUint8Array } from "../../src/bytes"
import { type Effect, flatMap, run, succeed } from "../../src/effect"
import { div, renderToString, span } from "../../src/html"
import {
  make,
  map,
  planSet,
  subscribe,
  transaction,
  unsubscribe,
} from "../../src/signal"
import { buffer, bufferCapacity, fromArray, runCollect } from "../../src/stream"

let binds: Effect<unknown, never, number> = succeed(0)
for (let index = 0; index < 100_000; index++)
  binds = flatMap(binds, (value) => succeed(value + 1))
const capacity = bufferCapacity(1)
if (capacity.tag !== "Right") throw new Error("invalid benchmark capacity")
const stream = buffer(
  capacity.value,
  fromArray(Array.from({ length: 128 }, (_, index) => index))
)
const bytes = fromUint8Array(new Uint8Array(4096).fill(42))
const html = div({
  children: Array.from({ length: 128 }, (_, index) =>
    span({ children: String(index) })
  ),
})
export const portableBenchmarks = suite("runtime", [
  inputSize(
    100_000,
    benchmark(
      "effect-bind",
      flatMap(binds, (value) => {
        if (blackBox(value) !== 100_000) throw new Error("bind result")
        return succeed(undefined)
      })
    )
  ),
  inputSize(
    128,
    benchmark(
      "stream-backpressure",
      flatMap(runCollect(stream), (values) => {
        if (blackBox(values).length !== 128) throw new Error("stream result")
        return succeed(undefined)
      })
    )
  ),
  inputSize(
    4096,
    benchmark("bytes-copy", () => {
      const copy = toUint8Array(bytes)
      copy[0] = 0
      if (blackBox(toUint8Array(bytes))[0] !== 42)
        throw new Error("Bytes alias")
      return undefined
    })
  ),
  inputSize(
    128,
    benchmark("ssr", () => {
      blackBox(renderToString(html))
      return undefined
    })
  ),
  inputSize(
    64,
    benchmark("signal-fanout-transaction", async (_, context) => {
      const source = await run(make(0), {}, context)
      if (source.kind !== "success") throw new Error("Signal creation")
      const subscriptions = []
      let deliveries = 0
      try {
        for (let index = 0; index < 64; index++) {
          const result = await run(
            subscribe(
              (value: number) => () => {
                blackBox(value)
                deliveries++
                return undefined
              },
              map((value: number) => value + index, source.value)
            ),
            {},
            context
          )
          if (result.kind !== "success") throw new Error("Signal subscribe")
          subscriptions.push(result.value)
        }
        deliveries = 0
        await run(
          transaction([planSet(1, source.value), planSet(2, source.value)]),
          {},
          context
        )
        if (deliveries !== 64) throw new Error(`Signal fanout ${deliveries}`)
      } finally {
        for (const subscription of subscriptions)
          await run(unsubscribe(subscription), {})
      }
    })
  ),
])

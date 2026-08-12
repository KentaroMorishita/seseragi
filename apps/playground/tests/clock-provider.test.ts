import { describe, expect, test } from "bun:test"
import {
  milliseconds,
  now,
  sleep,
  zeroDuration,
} from "../../../runtime/ts/src/clock"
import {
  createEffectExecution,
  isEffectCancellation,
  run,
} from "../../../runtime/ts/src/effect"
import {
  type ProviderEntry,
  providerRuntimeAbi,
  withProviderCancellation,
} from "../../../runtime/ts/src/provider"
import { createProviderClock } from "../../../runtime/ts/src/provider-clock"
import {
  defineProviderPackage,
  ProviderPackageLoader,
} from "../../../runtime/ts/src/provider-package"

let fixture = 0

function clockLoader(operations: ProviderEntry): ProviderPackageLoader {
  fixture += 1
  const provider = `fixture/runtime-bun#clock-${fixture}`
  const entry = defineProviderPackage({
    abi: providerRuntimeAbi,
    provider,
    service: "std/clock::Clock",
    targets: ["bun-process"],
    operations,
  })
  return new ProviderPackageLoader("bun-process", [
    {
      provider,
      service: "std/clock::Clock",
      target: "bun-process",
      module: "fixture/runtime-bun/clock",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({ provider: entry }),
    },
  ])
}

describe("Clock provider vertical slice", () => {
  test("keeps now and sleep cold until each Effect is run", async () => {
    let nowCalls = 0
    let sleepCalls = 0
    const loader = clockLoader({
      now: async () => {
        nowCalls += 1
        return { kind: "success", value: 42n }
      },
      sleep: async () => {
        sleepCalls += 1
        return { kind: "success", value: undefined }
      },
    })
    const selected = await loader.load(`fixture/runtime-bun#clock-${fixture}`)
    const environment = { clock: createProviderClock(selected) }
    const observe = now()
    const pause = sleep(zeroDuration())
    expect(nowCalls).toBe(0)
    expect(sleepCalls).toBe(0)

    expect((await run(observe, environment)).kind).toBe("success")
    expect((await run(pause, environment)).kind).toBe("success")
    expect(nowCalls).toBe(1)
    expect(sleepCalls).toBe(1)
    await loader.shutdown()
  })

  test("notifies a pending sleep once and preserves cancellation", async () => {
    let cancellations = 0
    let rejectSleep: (cause: unknown) => void = () => undefined
    const loader = clockLoader({
      now: async () => ({ kind: "success", value: 0n }),
      sleep: () => {
        const completion = new Promise<{
          kind: "success"
          value: undefined
        }>((_resolve, reject) => {
          rejectSleep = reject
        })
        return withProviderCancellation(completion, () => {
          cancellations += 1
          rejectSleep(new Error("cancelled timer"))
        })
      },
    })
    const selected = await loader.load(`fixture/runtime-bun#clock-${fixture}`)
    const duration = milliseconds(60_000)
    expect(duration.tag).toBe("Right")
    if (duration.tag === "Left") return
    const execution = createEffectExecution()
    const pending = run(
      sleep(duration.value),
      { clock: createProviderClock(selected) },
      execution.context
    ).catch((error: unknown) => error)

    const first = execution.cancel()
    const second = execution.cancel()
    expect(first).toBe(second)
    expect(isEffectCancellation(await pending)).toBe(true)
    await first
    expect(cancellations).toBe(1)
    await loader.shutdown()
  })
})

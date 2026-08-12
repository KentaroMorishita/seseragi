import {
  providerRuntimeAbi,
  withProviderCancellation,
} from "@seseragi/runtime/provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
} from "@seseragi/runtime/provider-package"

const maximumTimerMilliseconds = 2_147_483_647n

export type BunClockHost = Readonly<{
  monotonicNow: () => bigint
  setTimer: (callback: () => void, milliseconds: number) => unknown
  clearTimer: (timer: unknown) => void
}>

const liveHost: BunClockHost = Object.freeze({
  monotonicNow: () => process.hrtime.bigint(),
  setTimer: (callback, milliseconds) => setTimeout(callback, milliseconds),
  clearTimer: (timer) => clearTimeout(timer as ReturnType<typeof setTimeout>),
})

export function createBunClockProvider(
  host: BunClockHost = liveHost
): ProviderPackageEntry {
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: "seseragi/runtime-bun#clock",
    service: "std/clock::Clock",
    targets: ["bun-process"],
    operations: {
      async now() {
        return { kind: "success", value: host.monotonicNow() }
      },
      sleep(value) {
        if (typeof value !== "bigint" || value < 0n) {
          return Promise.reject(
            new TypeError("Clock sleep duration must be non-negative bigint")
          )
        }
        let timer: unknown
        let remaining = (value + 999_999n) / 1_000_000n
        let settled = false
        let rejectCompletion: (cause: unknown) => void = () => undefined
        const completion = new Promise<{
          kind: "success"
          value: undefined
        }>((resolve, reject) => {
          rejectCompletion = reject
          const schedule = (): void => {
            if (settled) return
            if (remaining === 0n) {
              settled = true
              resolve({ kind: "success", value: undefined })
              return
            }
            const chunk =
              remaining > maximumTimerMilliseconds
                ? maximumTimerMilliseconds
                : remaining
            remaining -= chunk
            timer = host.setTimer(schedule, Number(chunk))
          }
          schedule()
        })
        return withProviderCancellation(completion, () => {
          if (settled) return
          settled = true
          if (timer !== undefined) host.clearTimer(timer)
          rejectCompletion(new Error("Clock sleep cancelled"))
        })
      },
    },
  })
}

export const provider = createBunClockProvider()

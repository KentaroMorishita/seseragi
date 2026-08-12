import { providerRuntimeAbi, withProviderCancellation } from "../provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
} from "../provider-package"

const maximumTimerMilliseconds = 2_147_483_647n

export type BrowserClockHost = Readonly<{
  monotonicNow: () => bigint
  setTimer: (callback: () => void, milliseconds: number) => unknown
  clearTimer: (timer: unknown) => void
}>

const liveHost: BrowserClockHost = Object.freeze({
  monotonicNow: () => BigInt(Math.floor(performance.now() * 1_000_000)),
  setTimer: (callback, milliseconds) => setTimeout(callback, milliseconds),
  clearTimer: (timer) => clearTimeout(timer as ReturnType<typeof setTimeout>),
})

export function createBrowserClockProvider(
  host: BrowserClockHost = liveHost
): ProviderPackageEntry {
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: "seseragi/runtime-browser#clock",
    service: "std/clock::Clock",
    targets: ["browser"],
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

export const provider = createBrowserClockProvider()

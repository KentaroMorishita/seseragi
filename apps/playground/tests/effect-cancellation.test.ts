import { describe, expect, test } from "bun:test"
import {
  createEffectExecution,
  EffectCancellation,
  effectContextOf,
  fail,
  mapError,
  run,
  succeed,
} from "../../../runtime/ts/src/effect"
import {
  type ServiceResult,
  serviceEffect,
  serviceSuccess,
} from "../../../runtime/ts/src/service"
import type { EntryContract } from "../src/compiler/types"
import {
  type BrowserExecutionCompletion,
  startGeneratedModule,
} from "../src/runtime/browser-execution"

const consoleEntry: EntryContract = Object.freeze({
  environment: [Object.freeze({ field: "console", service: "console" })],
  failureRenderer: Object.freeze({ kind: "never" }),
})

function completed(
  completion: BrowserExecutionCompletion
): BrowserExecutionCompletion & { readonly kind: "completed" } {
  if (completion.kind === "completed") return completion
  throw new Error("expected a completed browser execution")
}

describe("Effect runtime cancellation", () => {
  test("cancels a pending host operation once without using the typed failure channel", async () => {
    const execution = createEffectExecution()
    let cleanups = 0
    let mapped = false
    const pending = serviceEffect(
      (_environment, context) =>
        new Promise<ServiceResult<never, string>>((resolve) => {
          context.onCancel(() => {
            cleanups += 1
            resolve(serviceSuccess("ignored"))
          })
        })
    )
    const result = run(
      mapError(() => {
        mapped = true
        return "should not be reached"
      }, pending),
      {},
      execution.context
    )

    const firstCancel = execution.cancel()
    const secondCancel = execution.cancel()

    expect(secondCancel).toBe(firstCancel)
    await firstCancel
    await expect(result).rejects.toBeInstanceOf(EffectCancellation)
    expect(cleanups).toBe(1)
    expect(mapped).toBe(false)
  })

  test("keeps a program's typed failure separate from host cancellation", async () => {
    await expect(run(fail("expected"), {})).resolves.toEqual({
      kind: "failure",
      error: "expected",
    })
  })

  test("keeps cancellation contexts isolated per root execution", async () => {
    const cancelled = createEffectExecution()
    const active = createEffectExecution()

    await cancelled.cancel()

    expect(cancelled.context.cancelled).toBe(true)
    expect(active.context.cancelled).toBe(false)
    await expect(
      run(succeed("still active"), {}, active.context)
    ).resolves.toEqual({
      kind: "success",
      value: "still active",
    })
  })

  test("preserves host defects instead of treating them as cancellation", async () => {
    const defect = new Error("host defect")
    const execution = createEffectExecution()

    await expect(
      run(
        () => {
          throw defect
        },
        {},
        execution.context
      )
    ).rejects.toBe(defect)
  })
})

describe("Browser execution cancellation lifecycle", () => {
  test("passes the root cancellation context through the browser environment", async () => {
    const key = "__seseragiEffectContextProbe"
    const probe = { sameContext: false }
    Object.assign(globalThis, { [key]: probe })
    try {
      const execution = await startGeneratedModule(
        `
          import { effectContextOf } from "@seseragi/runtime/effect"
          export const main = (_unit: undefined) => (environment, context) => {
            globalThis.__seseragiEffectContextProbe.sameContext =
              effectContextOf(environment) === context
            return undefined
          }
        `,
        Object.freeze({
          environment: [],
          failureRenderer: Object.freeze({ kind: "never" }),
        })
      )

      const result = completed(await execution.completion)

      expect(result.result).toEqual({ stdout: "", debug: "()" })
      expect(probe.sameContext).toBe(true)
      expect(effectContextOf({})).toBeUndefined()
    } finally {
      Reflect.deleteProperty(globalThis, key)
    }
  })

  test("settles a cancellation-unaware Run A before Run B and ignores A's late output", async () => {
    const first = await startGeneratedModule(
      `
        import { serviceEffect, serviceSuccess } from "@seseragi/runtime/service"
        export const main = (_unit: undefined) =>
          serviceEffect((environment) =>
            new Promise((resolve) => {
              setTimeout(() => {
                environment.console.println("A")
                resolve(serviceSuccess(undefined))
              }, 25)
            })
          )
      `,
      consoleEntry
    )

    const firstCancel = first.cancel()
    const secondCancel = first.cancel()
    expect(secondCancel).toBe(firstCancel)
    await firstCancel
    expect(await first.completion).toEqual({ kind: "cancelled" })

    const second = await startGeneratedModule(
      `
        import { println } from "@seseragi/runtime/console"
        export const main = (_unit: undefined) => println("B")
      `,
      consoleEntry
    )
    const secondResult = completed(await second.completion)

    expect(secondResult.result).toEqual({ stdout: "B", debug: "()" })
    await Bun.sleep(40)
    expect(await first.completion).toEqual({ kind: "cancelled" })
  })
})

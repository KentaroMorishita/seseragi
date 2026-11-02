import { describe, expect, test } from "bun:test"
import { fromUint8Array } from "../../../runtime/ts/src/bytes"
import {
  captureLimit,
  CloseChildStdin,
  command,
  ExecutablePath,
  runCaptured,
  runInherited,
  runStreaming,
  SearchPath,
  type ChildInput,
  type Executable,
} from "../../../runtime/ts/src/child-process"
import {
  fail,
  run,
  type Effect,
  type EffectContext,
} from "../../../runtime/ts/src/effect"
import { parse, render } from "../../../runtime/ts/src/path"
import {
  ProviderBoundaryDefect,
  type ProviderEntry,
  type ProviderSubscriptionObserver,
  providerRuntimeAbi,
} from "../../../runtime/ts/src/provider"
import { createProviderChildProcesses } from "../../../runtime/ts/src/provider-child-process"
import {
  defineProviderPackage,
  ProviderPackageLoader,
} from "../../../runtime/ts/src/provider-package"
import { fromEffect, runCollect } from "../../../runtime/ts/src/stream"

let fixture = 0

async function environment(operations: ProviderEntry) {
  fixture += 1
  const provider = `fixture/runtime-bun#child-process-${fixture}`
  const entry = defineProviderPackage({
    abi: providerRuntimeAbi,
    provider,
    service: "std/child-process::ChildProcesses",
    targets: ["bun-process"],
    operations,
  })
  const loader = new ProviderPackageLoader("bun-process", [
    {
      provider,
      service: "std/child-process::ChildProcesses",
      target: "bun-process",
      module: "fixture/runtime-bun/child-process",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({ provider: entry }),
    },
  ])
  return {
    loader,
    childProcesses: createProviderChildProcesses(await loader.load(provider)),
  }
}

function configuredCommand(executable: Executable = SearchPath("fixture")) {
  const configured = command(executable)
  if (configured.tag === "Left") {
    throw new Error("static child command must be valid")
  }
  return configured.value
}

function capturedEffect(executable?: Executable) {
  const limit = captureLimit(1024)
  if (limit.tag === "Left") {
    throw new Error("static child capture limit must be valid")
  }
  return runCaptured(
    limit.value,
    fromUint8Array(new Uint8Array()),
    configuredCommand(executable)
  )
}

describe("child-process provider boundary", () => {
  test("keeps decoded ExecutablePath spawn failures in both failure channels", async () => {
    const path = parse("/missing-child")
    if (path.tag === "Left") throw new Error("static path must be valid")
    const failure = {
      tag: "ChildSpawnFailed",
      value: {
        executable: { tag: "ExecutablePath", value: "/missing-child" },
        detail: "missing child",
      },
    }
    const selected = await environment({
      async runCaptured() {
        return { kind: "failure", failure }
      },
      async runInherited() {
        return { kind: "failure", failure }
      },
    })
    try {
      const executable = ExecutablePath(path.value)
      for (const result of [
        await run(capturedEffect(executable), selected),
        await run(runInherited(configuredCommand(executable)), selected),
      ]) {
        expect(result.kind).toBe("failure")
        if (
          result.kind !== "failure" ||
          result.error.tag !== "ChildSpawnFailed"
        ) {
          throw new Error("expected ChildSpawnFailed")
        }
        expect(result.error.value.detail).toBe("missing child")
        expect(result.error.value.executable.tag).toBe("ExecutablePath")
        if (result.error.value.executable.tag !== "ExecutablePath") {
          throw new Error("expected ExecutablePath")
        }
        expect(render(result.error.value.executable.value)).toBe(
          "/missing-child"
        )
      }
    } finally {
      await selected.loader.shutdown()
    }
  })

  test("lifts decoded input and child subscription failures and closes once", async () => {
    const input: Effect<unknown, string, ChildInput> = () => CloseChildStdin
    for (const [failure, expected] of [
      [
        { kind: "input", error: "broken input" },
        { tag: "Left", value: "broken input" },
      ],
      [
        { kind: "child", error: { tag: "ChildInputAfterClose" } },
        { tag: "Right", value: { tag: "ChildInputAfterClose" } },
      ],
    ] as const) {
      let closes = 0
      const selected = await environment({
        runStreaming(_request, observerValue) {
          const observer = observerValue as ProviderSubscriptionObserver
          return {
            demand() {
              observer.failure(failure)
            },
            unsubscribe() {
              closes += 1
            },
          }
        },
      })
      try {
        const result = await run(
          runCollect(runStreaming(fromEffect(input), configuredCommand())),
          selected
        )
        expect(result).toEqual({ kind: "failure", error: expected })
        expect(closes).toBe(1)
      } finally {
        await selected.loader.shutdown()
      }
    }
  })

  test("maps input open failures to Left before spawning", async () => {
    let registrations = 0
    const inputEffect: Effect<unknown, string, ChildInput> = () =>
      CloseChildStdin
    const input = {
      ...fromEffect(inputEffect),
      open(environment: unknown, context: EffectContext) {
        return fail("input open failed")(environment, context)
      },
    }
    const selected = await environment({
      runStreaming() {
        registrations += 1
        return { demand() {}, unsubscribe() {} }
      },
    })
    try {
      expect(
        await run(
          runCollect(runStreaming(input, configuredCommand())),
          selected
        )
      ).toEqual({
        kind: "failure",
        error: { tag: "Left", value: "input open failed" },
      })
      expect(registrations).toBe(0)
    } finally {
      await selected.loader.shutdown()
    }
  })

  test("preserves malformed streaming failures and raw throws as defects", async () => {
    const input: Effect<unknown, string, ChildInput> = () => CloseChildStdin
    for (const [method, value, stage] of [
      ["failure", { kind: "child", error: { tag: "FutureFailure" } }, "result"],
      ["failure", { kind: "input" }, "result"],
      [
        "defect",
        { tag: "Right", value: { tag: "ChildInputAfterClose" } },
        "call",
      ],
    ] as const) {
      let closes = 0
      const selected = await environment({
        runStreaming(_request, observerValue) {
          const observer = observerValue as ProviderSubscriptionObserver
          return {
            demand() {
              observer[method](value)
            },
            unsubscribe() {
              closes += 1
            },
          }
        },
      })
      try {
        const defect = await run(
          runCollect(runStreaming(fromEffect(input), configuredCommand())),
          selected
        ).catch((error: unknown) => error)
        expect(defect).toBeInstanceOf(ProviderBoundaryDefect)
        if (!(defect instanceof ProviderBoundaryDefect))
          throw new Error("expected provider defect")
        expect(defect.stage).toBe(stage)
        expect(defect.operation).toBe(
          "std/child-process::ChildProcesses#runStreaming"
        )
        expect(closes).toBe(1)
      } finally {
        await selected.loader.shutdown()
      }
    }
  })

  test("rejects unknown and malformed failure values as provider defects", async () => {
    for (const failure of [
      { tag: "FutureChildFailure", value: "not in the contract" },
      {
        tag: "ChildSpawnFailed",
        value: {
          executable: { tag: "SearchPath", value: "fixture" },
        },
      },
    ]) {
      const selected = await environment({
        async runCaptured() {
          return { kind: "failure", failure }
        },
      })
      const defect = await run(capturedEffect(), {
        childProcesses: selected.childProcesses,
      }).catch((error: unknown) => error)

      expect(defect).toBeInstanceOf(ProviderBoundaryDefect)
      if (!(defect instanceof ProviderBoundaryDefect)) {
        throw new Error("expected a child-process provider boundary defect")
      }
      expect(defect.stage).toBe("result")
      expect(defect.operation).toBe(
        "std/child-process::ChildProcesses#runCaptured"
      )
      await selected.loader.shutdown()
    }
  })
})

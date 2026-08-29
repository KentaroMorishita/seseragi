import { describe, expect, test } from "bun:test"
import { fromUint8Array } from "../../../runtime/ts/src/bytes"
import {
  captureLimit,
  command,
  runCaptured,
  SearchPath,
} from "../../../runtime/ts/src/child-process"
import { run } from "../../../runtime/ts/src/effect"
import {
  ProviderBoundaryDefect,
  type ProviderEntry,
  providerRuntimeAbi,
} from "../../../runtime/ts/src/provider"
import { createProviderChildProcesses } from "../../../runtime/ts/src/provider-child-process"
import {
  defineProviderPackage,
  ProviderPackageLoader,
} from "../../../runtime/ts/src/provider-package"

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

function capturedEffect() {
  const configured = command(SearchPath("fixture"))
  if (configured.tag === "Left") {
    throw new Error("static child command must be valid")
  }
  const limit = captureLimit(1024)
  if (limit.tag === "Left") {
    throw new Error("static child capture limit must be valid")
  }
  return runCaptured(
    limit.value,
    fromUint8Array(new Uint8Array()),
    configured.value
  )
}

describe("child-process provider boundary", () => {
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

import { describe, expect, test } from "bun:test"
import { createEffectExecution, run } from "../../../runtime/ts/src/effect"
import {
  close,
  filePath,
  openRead,
  read,
} from "../../../runtime/ts/src/filesystem"
import {
  ProviderBoundaryDefect,
  type ProviderEntry,
  providerRuntimeAbi,
} from "../../../runtime/ts/src/provider"
import { createProviderFileSystem } from "../../../runtime/ts/src/provider-filesystem"
import {
  defineProviderPackage,
  ProviderPackageLoader,
} from "../../../runtime/ts/src/provider-package"

let fixture = 0

async function environment(operations: ProviderEntry) {
  fixture += 1
  const provider = `fixture/runtime-bun#filesystem-${fixture}`
  const entry = defineProviderPackage({
    abi: providerRuntimeAbi,
    provider,
    service: "std/fs::FileSystem",
    targets: ["bun-process"],
    operations,
  })
  const loader = new ProviderPackageLoader("bun-process", [
    {
      provider,
      service: "std/fs::FileSystem",
      target: "bun-process",
      module: "fixture/runtime-bun/filesystem",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({ provider: entry }),
    },
  ])
  return {
    loader,
    fileSystem: createProviderFileSystem(await loader.load(provider)),
  }
}

describe("filesystem provider vertical slice", () => {
  test("keeps open/read cold and closes once on cancellation", async () => {
    let openCalls = 0
    let readCalls = 0
    let closeCalls = 0
    const token = Object.freeze({ fd: 1 })
    const selected = await environment({
      async openRead() {
        openCalls += 1
        return { kind: "success", value: token }
      },
      async read() {
        readCalls += 1
        return { kind: "success", value: new Uint8Array([1, 2, 3]) }
      },
      async close() {
        closeCalls += 1
        return { kind: "success", value: undefined }
      },
    })
    const execution = createEffectExecution()
    const acquire = openRead(filePath("fixture.bin"))
    expect(openCalls).toBe(0)
    const opened = await run(
      acquire,
      { fileSystem: selected.fileSystem },
      execution.context
    )
    expect(opened.kind).toBe("success")
    expect(openCalls).toBe(1)
    if (opened.kind !== "success") return

    const chunk = await run(
      read(opened.value, 3),
      { fileSystem: selected.fileSystem },
      execution.context
    )
    expect(chunk).toEqual({ kind: "success", value: new Uint8Array([1, 2, 3]) })
    expect(readCalls).toBe(1)

    const first = execution.cancel()
    const second = execution.cancel()
    expect(first).toBe(second)
    await first
    expect(closeCalls).toBe(1)
    expect(
      (
        await run(close(opened.value), {
          fileSystem: selected.fileSystem,
        })
      ).kind
    ).toBe("success")
    expect(closeCalls).toBe(1)
    await selected.loader.shutdown()
    expect(closeCalls).toBe(1)
  })

  test("rejects a handle owned by another provider before host read", async () => {
    let foreignReads = 0
    const operations = (onRead: () => void): ProviderEntry => ({
      openRead: async () => ({ kind: "success", value: {} }),
      read: async () => {
        onRead()
        return { kind: "success", value: new Uint8Array() }
      },
      close: async () => ({ kind: "success", value: undefined }),
    })
    const owner = await environment(operations(() => undefined))
    const foreign = await environment(
      operations(() => {
        foreignReads += 1
      })
    )
    const opened = await run(openRead(filePath("fixture.bin")), {
      fileSystem: owner.fileSystem,
    })
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return

    const defect = await run(read(opened.value, 1), {
      fileSystem: foreign.fileSystem,
    }).catch((error: unknown) => error)
    expect(defect).toBeInstanceOf(ProviderBoundaryDefect)
    if (!(defect instanceof ProviderBoundaryDefect)) {
      throw new Error("expected a filesystem provider boundary defect")
    }
    expect(defect.stage).toBe("input")
    expect(foreignReads).toBe(0)
    await run(close(opened.value), { fileSystem: owner.fileSystem })
    await foreign.loader.shutdown()
    await owner.loader.shutdown()
  })
})

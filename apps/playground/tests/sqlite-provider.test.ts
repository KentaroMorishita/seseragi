import { describe, expect, test } from "bun:test"
import { createEffectExecution, run } from "../../../runtime/ts/src/effect"
import {
  type ProviderEntry,
  providerRuntimeAbi,
} from "../../../runtime/ts/src/provider"
import {
  defineProviderPackage,
  ProviderPackageLoader,
} from "../../../runtime/ts/src/provider-package"
import { createProviderSqlite } from "../../../runtime/ts/src/provider-sqlite"
import {
  openMemory,
  query,
  type SqliteDecoder,
  transaction,
} from "../../../runtime/ts/src/sqlite"

let fixture = 0

async function environment(operations: ProviderEntry) {
  fixture += 1
  const provider = `fixture/runtime-sqlite#bun-${fixture}`
  const entry = defineProviderPackage({
    abi: providerRuntimeAbi,
    provider,
    service: "seseragi/sqlite::Sqlite",
    targets: ["bun-process"],
    operations,
  })
  const loader = new ProviderPackageLoader("bun-process", [
    {
      provider,
      service: "seseragi/sqlite::Sqlite",
      target: "bun-process",
      module: "fixture/runtime-sqlite/bun",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({ provider: entry }),
    },
  ])
  return {
    loader,
    sqlite: createProviderSqlite(await loader.load(provider)),
  }
}

describe("SQLite provider vertical slice", () => {
  test("keeps busy failures typed and copies row Bytes", async () => {
    const database = {}
    const bytes = new Uint8Array([1, 2, 3])
    const selected = await environment({
      openMemory: async () => ({ kind: "success", value: database }),
      query: async (value) => {
        const sql = (value as { statement: { sql: string } }).statement.sql
        return sql === "busy"
          ? {
              kind: "failure",
              failure: {
                tag: "QueryFailed",
                operation: "query",
                code: "SQLITE_BUSY",
                message: "database is locked",
              },
            }
          : {
              kind: "success",
              value: { rows: [{ id: 1, payload: bytes }] },
            }
      },
      close: async () => ({ kind: "success", value: undefined }),
    })
    const runtimeEnvironment = { sqlite: selected.sqlite }
    const execution = createEffectExecution()
    const opened = await run(
      openMemory(1_000),
      runtimeEnvironment,
      execution.context
    )
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return
    const rowDecoder: SqliteDecoder<Readonly<Record<string, unknown>>> = (
      row
    ) => ({ tag: "Decoded", value: row })
    const rows = await run(
      query(opened.value, { sql: "select", values: [] }, rowDecoder),
      runtimeEnvironment,
      execution.context
    )
    bytes[0] = 99
    expect(rows).toEqual({
      kind: "success",
      value: {
        rows: [{ id: 1, payload: new Uint8Array([1, 2, 3]) }],
      },
    })
    const busy = await run(
      query(opened.value, { sql: "busy", values: [] }, rowDecoder),
      runtimeEnvironment,
      execution.context
    )
    expect(busy).toEqual({
      kind: "failure",
      error: {
        tag: "BusyFailure",
        value: {
          operation: "query",
          code: "SQLITE_BUSY",
          message: "database is locked",
        },
      },
    })
    await execution.close()
    await selected.loader.shutdown()
  })

  test("rolls back an active transaction before database cleanup on cancel", async () => {
    const trace: string[] = []
    const selected = await environment({
      openMemory: async () => ({ kind: "success", value: {} }),
      begin: async () => {
        trace.push("begin")
        return { kind: "success", value: {} }
      },
      rollback: async () => {
        trace.push("rollback")
        return { kind: "success", value: undefined }
      },
      close: async () => {
        trace.push("close")
        return { kind: "success", value: undefined }
      },
    })
    const runtimeEnvironment = { sqlite: selected.sqlite }
    const execution = createEffectExecution()
    const opened = await run(
      openMemory(1_000),
      runtimeEnvironment,
      execution.context
    )
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return

    const pending = run(
      transaction(opened.value, () => () => {
        trace.push("program")
        return new Promise(() => undefined)
      }),
      runtimeEnvironment,
      execution.context
    )
    const cancellation = pending.then(
      () => "unexpected-success",
      (error: unknown) =>
        error instanceof Error ? error.name : "unexpected-rejection"
    )
    for (
      let attempt = 0;
      attempt < 100 && !trace.includes("program");
      attempt += 1
    )
      await Bun.sleep(1)
    expect(trace).toContain("program")
    await execution.cancel()
    expect(await cancellation).toBe("EffectCancellation")
    expect(trace).toEqual(["begin", "program", "rollback", "close"])
    await selected.loader.shutdown()
    expect(trace).toEqual(["begin", "program", "rollback", "close"])
  })

  test("retains rollback ownership after a typed commit failure", async () => {
    const trace: string[] = []
    const selected = await environment({
      openMemory: async () => ({ kind: "success", value: {} }),
      begin: async () => {
        trace.push("begin")
        return { kind: "success", value: {} }
      },
      commit: async () => {
        trace.push("commit")
        return {
          kind: "failure",
          failure: {
            tag: "QueryFailed",
            operation: "commit",
            code: "SQLITE_IOERR",
            message: "commit failed",
          },
        }
      },
      rollback: async () => {
        trace.push("rollback")
        return { kind: "success", value: undefined }
      },
      close: async () => {
        trace.push("close")
        return { kind: "success", value: undefined }
      },
    })
    const execution = createEffectExecution()
    const opened = await selected.sqlite.openMemory(1_000, execution.context)
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return
    const begun = await selected.sqlite.begin(opened.value, execution.context)
    expect(begun.kind).toBe("success")
    if (begun.kind !== "success") return
    expect(
      (await selected.sqlite.commit(begun.value, execution.context)).kind
    ).toBe("failure")
    expect(trace).toEqual(["begin", "commit"])

    await execution.close()
    expect(trace).toEqual(["begin", "commit", "rollback", "close"])
    await selected.loader.shutdown()
  })

  test("releases successful transactions and rolls back active children in reverse", async () => {
    const trace: string[] = []
    let nextTransaction = 0
    const selected = await environment({
      openMemory: async () => ({ kind: "success", value: {} }),
      begin: async () => {
        nextTransaction += 1
        trace.push(`begin-${nextTransaction}`)
        return { kind: "success", value: { id: nextTransaction } }
      },
      commit: async (value) => {
        trace.push(`commit-${(value as { id: number }).id}`)
        return { kind: "success", value: undefined }
      },
      rollback: async (value) => {
        trace.push(`rollback-${(value as { id: number }).id}`)
        return { kind: "success", value: undefined }
      },
      close: async () => {
        trace.push("close")
        return { kind: "success", value: undefined }
      },
    })
    const execution = createEffectExecution()
    const opened = await selected.sqlite.openMemory(1_000, execution.context)
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return
    const first = await selected.sqlite.begin(opened.value, execution.context)
    const second = await selected.sqlite.begin(opened.value, execution.context)
    const third = await selected.sqlite.begin(opened.value, execution.context)
    const fourth = await selected.sqlite.begin(opened.value, execution.context)
    if (
      first.kind !== "success" ||
      second.kind !== "success" ||
      third.kind !== "success" ||
      fourth.kind !== "success"
    )
      throw new Error("fixture transaction failed to begin")

    expect(
      (await selected.sqlite.commit(first.value, execution.context)).kind
    ).toBe("success")
    expect(
      (await selected.sqlite.rollback(second.value, execution.context)).kind
    ).toBe("success")
    expect(
      (await selected.sqlite.close(opened.value, execution.context)).kind
    ).toBe("success")
    await execution.close()
    expect(trace).toEqual([
      "begin-1",
      "begin-2",
      "begin-3",
      "begin-4",
      "commit-1",
      "rollback-2",
      "rollback-4",
      "rollback-3",
      "close",
    ])
    await selected.loader.shutdown()
  })

  test("serializes cancellation rollback behind an in-flight commit", async () => {
    const trace: string[] = []
    let failCommit: () => void = () => undefined
    const selected = await environment({
      openMemory: async () => ({ kind: "success", value: {} }),
      begin: async () => ({ kind: "success", value: {} }),
      commit: () =>
        new Promise((resolve) => {
          trace.push("commit")
          failCommit = () =>
            resolve({
              kind: "failure",
              failure: {
                tag: "QueryFailed",
                operation: "commit",
                code: "SQLITE_IOERR",
                message: "commit failed",
              },
            })
        }),
      rollback: async () => {
        trace.push("rollback")
        return { kind: "success", value: undefined }
      },
      close: async () => {
        trace.push("close")
        return { kind: "success", value: undefined }
      },
    })
    const runtimeEnvironment = { sqlite: selected.sqlite }
    const execution = createEffectExecution()
    const opened = await run(
      openMemory(1_000),
      runtimeEnvironment,
      execution.context
    )
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return
    const pending = run(
      transaction(opened.value, () => async () => "done"),
      runtimeEnvironment,
      execution.context
    )
    const cancellation = pending.then(
      () => "unexpected-success",
      (error: unknown) =>
        error instanceof Error ? error.name : "unexpected-rejection"
    )
    for (let attempt = 0; attempt < 100 && trace.length === 0; attempt += 1)
      await Bun.sleep(1)
    expect(trace).toEqual(["commit"])

    const cancelling = execution.cancel()
    failCommit()
    await cancelling
    expect(await cancellation).toBe("EffectCancellation")
    expect(trace).toEqual(["commit", "rollback", "close"])
    await selected.loader.shutdown()
  })

  test("preserves cleanup primary and suppressed defects without retrying rollback", async () => {
    const trace: string[] = []
    const selected = await environment({
      openMemory: async () => ({ kind: "success", value: {} }),
      begin: async () => ({ kind: "success", value: {} }),
      rollback: async () => {
        trace.push("rollback")
        return {
          kind: "failure",
          failure: {
            tag: "QueryFailed",
            operation: "rollback",
            code: "SQLITE_IOERR",
            message: "rollback failed",
          },
        }
      },
      close: async () => {
        trace.push("close")
        return { kind: "success", value: undefined }
      },
    })
    const execution = createEffectExecution()
    const opened = await selected.sqlite.openMemory(1_000, execution.context)
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return
    expect(
      (await selected.sqlite.begin(opened.value, execution.context)).kind
    ).toBe("success")

    let defect: unknown
    try {
      await execution.close()
    } catch (error) {
      defect = error
    }
    expect(defect).toBeInstanceOf(Error)
    expect((defect as Error).message).toBe(
      "SQLite cleanup failed: rollback failed"
    )
    const suppressed = (defect as { suppressed?: ReadonlyArray<unknown> })
      .suppressed
    expect(suppressed).toHaveLength(1)
    expect((suppressed?.[0] as Error).message).toBe(
      "SQLite cleanup failed: rollback failed"
    )
    expect(trace).toEqual(["rollback", "close"])
    await selected.loader.shutdown()
  })
})

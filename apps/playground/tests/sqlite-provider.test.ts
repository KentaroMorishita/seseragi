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
})

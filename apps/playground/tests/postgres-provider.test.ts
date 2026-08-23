import { describe, expect, test } from "bun:test"
import { createEffectExecution, run } from "../../../runtime/ts/src/effect"
import {
  fetchPostgresRows,
  openPostgresCursor,
  openPostgresPool,
  queryPostgres,
} from "../../../runtime/ts/src/postgres"
import {
  type ProviderEntry,
  providerRuntimeAbi,
} from "../../../runtime/ts/src/provider"
import {
  defineProviderPackage,
  ProviderPackageLoader,
} from "../../../runtime/ts/src/provider-package"
import { createProviderPostgres } from "../../../runtime/ts/src/provider-postgres"

let fixture = 0

async function environment(operations: ProviderEntry) {
  fixture += 1
  const provider = `fixture/runtime-postgres#pg-${fixture}`
  const entry = defineProviderPackage({
    abi: providerRuntimeAbi,
    provider,
    service: "acme/postgres::Postgres",
    targets: ["bun-process"],
    operations,
  })
  const loader = new ProviderPackageLoader("bun-process", [
    {
      provider,
      service: "acme/postgres::Postgres",
      target: "bun-process",
      module: "fixture/runtime-postgres/pg",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({ provider: entry }),
    },
  ])
  return {
    loader,
    postgres: createProviderPostgres(await loader.load(provider)),
  }
}

describe("PostgreSQL provider vertical slice", () => {
  test("keeps driver failure typed and copies row Bytes", async () => {
    const pool = {}
    const bytes = new Uint8Array([1, 2, 3])
    const selected = await environment({
      openPool: async () => ({ kind: "success", value: pool }),
      query: async (value) => {
        const text = (value as { query: { text: string } }).query.text
        return text === "fail"
          ? {
              kind: "failure",
              failure: {
                tag: "QueryFailed",
                operation: "query",
                code: "23505",
                message: "duplicate key",
              },
            }
          : { kind: "success", value: [{ id: 1, payload: bytes }] }
      },
      openCursor: async () => ({ kind: "success", value: {} }),
      fetch: async () => ({ kind: "success", value: [] }),
      closeCursor: async () => ({ kind: "success", value: undefined }),
      closePool: async () => ({ kind: "success", value: undefined }),
    })
    const runtimeEnvironment = { postgres: selected.postgres }
    const execution = createEffectExecution()
    const opened = await run(
      openPostgresPool({ connectionString: "postgres://fixture/test" }),
      runtimeEnvironment,
      execution.context
    )
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return
    const rows = await run(
      queryPostgres(opened.value, { text: "ok", values: [] }),
      runtimeEnvironment,
      execution.context
    )
    bytes[0] = 99
    expect(rows).toEqual({
      kind: "success",
      value: [{ id: 1, payload: new Uint8Array([1, 2, 3]) }],
    })
    const failed = await run(
      queryPostgres(opened.value, { text: "fail", values: [] }),
      runtimeEnvironment,
      execution.context
    )
    expect(failed).toEqual({
      kind: "failure",
      error: {
        tag: "QueryFailed",
        operation: "query",
        code: "23505",
        message: "duplicate key",
      },
    })
    await execution.close()
    await selected.loader.shutdown()
  })

  test("closes cursor before pool once on cancellation", async () => {
    const trace: string[] = []
    const selected = await environment({
      openPool: async () => ({ kind: "success", value: {} }),
      query: async () => ({ kind: "success", value: [] }),
      openCursor: async () => ({ kind: "success", value: {} }),
      fetch: async () => ({ kind: "success", value: [{ id: 1 }] }),
      closeCursor: async () => {
        trace.push("cursor")
        return { kind: "success", value: undefined }
      },
      closePool: async () => {
        trace.push("pool")
        return { kind: "success", value: undefined }
      },
    })
    const runtimeEnvironment = { postgres: selected.postgres }
    const execution = createEffectExecution()
    const opened = await run(
      openPostgresPool({ connectionString: "postgres://fixture/test" }),
      runtimeEnvironment,
      execution.context
    )
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return
    const cursor = await run(
      openPostgresCursor(opened.value, { text: "select 1", values: [] }),
      runtimeEnvironment,
      execution.context
    )
    expect(cursor.kind).toBe("success")
    if (cursor.kind !== "success") return
    expect(
      (
        await run(
          fetchPostgresRows(cursor.value, 1),
          runtimeEnvironment,
          execution.context
        )
      ).kind
    ).toBe("success")

    await execution.cancel()
    expect(trace).toEqual(["cursor", "pool"])
    await selected.loader.shutdown()
    expect(trace).toEqual(["cursor", "pool"])
  })
})

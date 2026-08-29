import { describe, expect, test } from "bun:test"
import { createEffectExecution, run } from "../../../runtime/ts/src/effect"
import {
  fetch,
  openCursor,
  openPool,
  type PostgresDecoder,
  query,
  transaction,
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
    service: "seseragi/postgres::Postgres",
    targets: ["bun-process"],
    operations,
  })
  const loader = new ProviderPackageLoader("bun-process", [
    {
      provider,
      service: "seseragi/postgres::Postgres",
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
          : {
              kind: "success",
              value: {
                rows: [{ id: 1, payload: bytes }],
                rowCount: 1,
                command: "SELECT",
              },
            }
      },
      openCursor: async () => ({ kind: "success", value: {} }),
      fetch: async () => ({ kind: "success", value: [] }),
      closeCursor: async () => ({ kind: "success", value: undefined }),
      closePool: async () => ({ kind: "success", value: undefined }),
    })
    const runtimeEnvironment = { postgres: selected.postgres }
    const execution = createEffectExecution()
    const opened = await run(
      openPool({
        connectionString: "postgres://fixture/test",
        maxConnections: 4,
      }),
      runtimeEnvironment,
      execution.context
    )
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return
    const rowDecoder: PostgresDecoder<Readonly<Record<string, unknown>>> = (
      row
    ) => ({ tag: "Decoded", value: row })
    const rows = await run(
      query(opened.value, { text: "ok", values: [] }, rowDecoder),
      runtimeEnvironment,
      execution.context
    )
    bytes[0] = 99
    expect(rows).toEqual({
      kind: "success",
      value: {
        rows: [{ id: 1, payload: new Uint8Array([1, 2, 3]) }],
        rowCount: 1,
        command: "SELECT",
      },
    })
    const failed = await run(
      query(opened.value, { text: "fail", values: [] }, rowDecoder),
      runtimeEnvironment,
      execution.context
    )
    expect(failed).toEqual({
      kind: "failure",
      error: {
        tag: "DriverFailure",
        value: {
          operation: "query",
          code: "23505",
          message: "duplicate key",
        },
      },
    })
    await execution.close()
    await selected.loader.shutdown()
  })

  test("closes cursor before pool once on cancellation", async () => {
    const trace: string[] = []
    const selected = await environment({
      openPool: async () => ({ kind: "success", value: {} }),
      query: async () => ({
        kind: "success",
        value: { rows: [], rowCount: 0, command: "SELECT" },
      }),
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
      openPool({
        connectionString: "postgres://fixture/test",
        maxConnections: 4,
      }),
      runtimeEnvironment,
      execution.context
    )
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return
    const cursor = await run(
      openCursor({ text: "select 1", values: [] }, opened.value),
      runtimeEnvironment,
      execution.context
    )
    expect(cursor.kind).toBe("success")
    if (cursor.kind !== "success") return
    expect(
      (
        await run(
          fetch(1, (row) => ({ tag: "Decoded", value: row }), cursor.value),
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

  test("retains rollback ownership after a typed commit failure", async () => {
    const trace: string[] = []
    const selected = await environment({
      openPool: async () => ({ kind: "success", value: {} }),
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
            code: "40001",
            message: "commit failed",
          },
        }
      },
      rollback: async () => {
        trace.push("rollback")
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
      openPool({
        connectionString: "postgres://fixture/test",
        maxConnections: 4,
      }),
      runtimeEnvironment,
      execution.context
    )
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return

    const result = await run(
      transaction(opened.value, () => async () => {
        trace.push("program")
        return "done"
      }),
      runtimeEnvironment,
      execution.context
    )
    expect(result.kind).toBe("failure")
    expect(trace).toEqual(["begin", "program", "commit", "rollback"])

    await execution.close()
    expect(trace).toEqual(["begin", "program", "commit", "rollback", "pool"])
    await selected.loader.shutdown()
  })

  test("releases successful transactions and rolls back active children in reverse", async () => {
    const trace: string[] = []
    let nextTransaction = 0
    const selected = await environment({
      openPool: async () => ({ kind: "success", value: {} }),
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
      closePool: async () => {
        trace.push("pool")
        return { kind: "success", value: undefined }
      },
    })
    const execution = createEffectExecution()
    const opened = await selected.postgres.openPool(
      {
        connectionString: "postgres://fixture/test",
        maxConnections: 4,
      },
      execution.context
    )
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return
    const first = await selected.postgres.begin(opened.value, execution.context)
    const second = await selected.postgres.begin(
      opened.value,
      execution.context
    )
    const third = await selected.postgres.begin(opened.value, execution.context)
    const fourth = await selected.postgres.begin(
      opened.value,
      execution.context
    )
    if (
      first.kind !== "success" ||
      second.kind !== "success" ||
      third.kind !== "success" ||
      fourth.kind !== "success"
    )
      throw new Error("fixture transaction failed to begin")

    expect(
      (await selected.postgres.commit(first.value, execution.context)).kind
    ).toBe("success")
    expect(
      (await selected.postgres.rollback(second.value, execution.context)).kind
    ).toBe("success")
    expect(
      (await selected.postgres.closePool(opened.value, execution.context)).kind
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
      "pool",
    ])
    await selected.loader.shutdown()
  })

  test("serializes cancellation rollback behind an in-flight commit", async () => {
    const trace: string[] = []
    let failCommit: () => void = () => undefined
    const selected = await environment({
      openPool: async () => ({ kind: "success", value: {} }),
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
                code: "40001",
                message: "commit failed",
              },
            })
        }),
      rollback: async () => {
        trace.push("rollback")
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
      openPool({
        connectionString: "postgres://fixture/test",
        maxConnections: 4,
      }),
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
    expect(trace).toEqual(["commit", "rollback", "pool"])
    await selected.loader.shutdown()
  })

  test("preserves cleanup primary and suppressed defects without retrying rollback", async () => {
    const trace: string[] = []
    const selected = await environment({
      openPool: async () => ({ kind: "success", value: {} }),
      begin: async () => ({ kind: "success", value: {} }),
      rollback: async () => {
        trace.push("rollback")
        return {
          kind: "failure",
          failure: {
            tag: "QueryFailed",
            operation: "rollback",
            code: "08006",
            message: "rollback failed",
          },
        }
      },
      closePool: async () => {
        trace.push("pool")
        return { kind: "success", value: undefined }
      },
    })
    const execution = createEffectExecution()
    const opened = await selected.postgres.openPool(
      {
        connectionString: "postgres://fixture/test",
        maxConnections: 4,
      },
      execution.context
    )
    expect(opened.kind).toBe("success")
    if (opened.kind !== "success") return
    expect(
      (await selected.postgres.begin(opened.value, execution.context)).kind
    ).toBe("success")

    let defect: unknown
    try {
      await execution.close()
    } catch (error) {
      defect = error
    }
    expect(defect).toBeInstanceOf(Error)
    expect((defect as Error).message).toBe(
      "PostgreSQL cleanup failed: rollback failed"
    )
    const suppressed = (defect as { suppressed?: ReadonlyArray<unknown> })
      .suppressed
    expect(suppressed).toHaveLength(1)
    expect((suppressed?.[0] as Error).message).toBe(
      "PostgreSQL cleanup failed: rollback failed"
    )
    expect(trace).toEqual(["rollback", "pool"])
    await selected.loader.shutdown()
  })
})

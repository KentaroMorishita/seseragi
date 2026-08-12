import { createEffectExecution, run } from "@seseragi/runtime/effect"
import { ProviderBoundaryDefect } from "@seseragi/runtime/provider"
import { assertProviderConformanceCase } from "@seseragi/runtime/provider-conformance"
import { ProviderPackageLoader } from "@seseragi/runtime/provider-package"
import { createProviderPostgres } from "@seseragi/runtime/provider-postgres"
import {
  createPostgresProvider,
  type DriverPool,
  type PostgresDriver,
} from "seseragi/runtime-postgres/adapter"
import {
  closeFixturePool,
  fetchFixtureRows,
  openFixtureCursor,
  openFixturePool,
  queryFixture,
} from "./postgres-application.ts"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

const trace: string[] = []
const rows = Object.freeze([
  Object.freeze({ id: 1, name: "Ada" }),
  Object.freeze({ id: 2, name: "Grace" }),
])
const driver: PostgresDriver = Object.freeze({
  createPool(connectionString) {
    assert(
      connectionString.startsWith("postgres://"),
      "connection string must cross"
    )
    const pool: DriverPool = {
      async query(text) {
        if (text === "fail") {
          throw Object.assign(new Error("duplicate key"), { code: "23505" })
        }
        if (text === "invalid-row") {
          return { rows: [{ createdAt: new Date() }] }
        }
        return { rows }
      },
      async connect() {
        return {
          release() {
            trace.push("client-release")
          },
        }
      },
      async end() {
        trace.push("pool-end")
      },
    }
    return pool
  },
  openCursor() {
    let offset = 0
    return {
      async read(limit) {
        const chunk = rows.slice(offset, offset + limit)
        offset += chunk.length
        return chunk
      },
      async close() {
        trace.push("cursor-close")
      },
    }
  },
})

const provider = requiredEnvironment("SESERAGI_POSTGRES_PROVIDER")
const service = requiredEnvironment("SESERAGI_POSTGRES_SERVICE")
const module = requiredEnvironment("SESERAGI_POSTGRES_MODULE")
const exportName = requiredEnvironment("SESERAGI_POSTGRES_EXPORT")
const target = requiredEnvironment("SESERAGI_POSTGRES_TARGET") as
  | "bun-process"
  | "node-process"
const entry = createPostgresProvider(driver)
const loader = new ProviderPackageLoader(target, [
  {
    provider,
    service,
    target,
    module,
    exportName,
    loadMode: "lazy",
    importModule: async () => Object.freeze({ provider: entry }),
    source: { path: "src/main.ssrg", start: 0, end: 10 },
  },
])
const environment = Object.freeze({
  postgres: createProviderPostgres(await loader.load(provider)),
})
const execution = createEffectExecution()
const opened = await run(
  openFixturePool("postgres://fixture/test"),
  environment,
  execution.context
)
assert(opened.kind === "success", "PostgreSQL pool must open")

const queried = await run(
  queryFixture(opened.value),
  environment,
  execution.context
)
assert(queried.kind === "success", "PostgreSQL query must succeed")
assert(
  JSON.stringify(queried.value) === JSON.stringify(rows),
  "PostgreSQL rows must cross the package boundary"
)
assertProviderConformanceCase({ id: "success", terminal: queried.kind })
const failed = await run(queryFixture(opened.value, "fail"), environment)
assert(failed.kind === "failure", "driver failure must stay typed")
if (failed.kind === "failure") {
  assert(failed.error.code === "23505", "driver error code must be preserved")
}
assertProviderConformanceCase({
  id: "typed-failure",
  terminal: failed.kind === "failure" ? "typed-failure" : failed.kind,
})
const defect = await run(
  queryFixture(opened.value, "invalid-row"),
  environment
).catch((error: unknown) => error)
assert(
  defect instanceof ProviderBoundaryDefect && defect.stage === "result",
  "invalid driver row must be a result boundary defect"
)
assertProviderConformanceCase({
  id: "invalid-value",
  boundary: "result",
  terminal: defect instanceof ProviderBoundaryDefect ? "defect" : "success",
  leakedToApplication: !(defect instanceof ProviderBoundaryDefect),
})

const cursor = await run(
  openFixtureCursor(opened.value),
  environment,
  execution.context
)
assert(cursor.kind === "success", "PostgreSQL cursor must open")
const first = await run(
  fetchFixtureRows(cursor.value, 1),
  environment,
  execution.context
)
assert(
  first.kind === "success" && first.value.length === 1,
  "cursor demand must bound rows"
)
await execution.cancel()
assert(
  JSON.stringify(trace) ===
    JSON.stringify(["cursor-close", "client-release", "pool-end"]),
  "cursor and connection must close before the pool"
)
assertProviderConformanceCase({
  id: "cancellation",
  terminal: "cancellation",
  notifications: 1,
  lateCompletion: "discarded",
})
assert(
  (await run(closeFixturePool(opened.value), environment)).kind === "success",
  "pool close must be idempotent after cancellation"
)
await loader.shutdown()
assertProviderConformanceCase({
  id: "cleanup",
  acquired: 3,
  released: trace.length,
  active: 3 - trace.length,
})
assertProviderConformanceCase({
  id: "leak",
  activeAfterCleanup: 3 - trace.length,
})
process.stdout.write(`PostgreSQL provider probe passed: ${target}\n`)

function requiredEnvironment(name: string): string {
  const value = process.env[name]
  if (value === undefined || value.length === 0)
    throw new Error(`missing ${name}`)
  return value
}

import {
  createEffectExecution,
  isEffectCancellation,
  run,
} from "@seseragi/runtime/effect"
import { ProviderBoundaryDefect } from "@seseragi/runtime/provider"
import { assertProviderConformanceCase } from "@seseragi/runtime/provider-conformance"
import { ProviderPackageLoader } from "@seseragi/runtime/provider-package"
import { createProviderSqlite } from "@seseragi/runtime/provider-sqlite"
import {
  bool,
  close,
  int,
  openMemory,
  query,
  type SqliteDecoder,
  type SqliteRow,
  string,
  type SqliteTransactionProgram,
  transaction,
  transactionExecute,
  transactionQuery,
} from "@seseragi/runtime/sqlite"
import {
  createSqliteProvider,
  type DriverDatabase,
} from "seseragi/runtime-sqlite/adapter"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

const trace: string[] = []
const driverBytes = new Uint8Array([1, 2, 3])
let failCommit = false
const database: DriverDatabase = Object.freeze({
  query(statement) {
    if (statement.sql === "busy") {
      throw Object.assign(new Error("database is locked"), {
        code: "SQLITE_BUSY",
      })
    }
    if (statement.sql === "invalid-row") {
      return [{ createdAt: new Date() }]
    }
    if (statement.sql === "decode-failure") {
      return [{ name: 42n, active: 2n }]
    }
    return [{ id: 1n, name: "Ada", active: 1n, payload: driverBytes }]
  },
  execute(statement) {
    trace.push(statement.sql)
    return { changes: 1n, lastInsertRowId: 1n }
  },
  beginImmediate() {
    trace.push("begin")
  },
  commit() {
    trace.push("commit")
    if (failCommit) {
      throw Object.assign(new Error("database is locked"), {
        code: "SQLITE_BUSY",
      })
    }
  },
  rollback() {
    trace.push("rollback")
  },
  close() {
    trace.push("close")
  },
})
const entry = createSqliteProvider({
  openMemory(busyTimeoutMillis) {
    assert(busyTimeoutMillis === 1_000, "busy timeout must cross")
    return database
  },
  openFile() {
    throw new Error("file database is not used by this probe")
  },
})
const provider = requiredEnvironment("SESERAGI_SQLITE_PROVIDER")
const service = requiredEnvironment("SESERAGI_SQLITE_SERVICE")
const module = requiredEnvironment("SESERAGI_SQLITE_MODULE")
const exportName = requiredEnvironment("SESERAGI_SQLITE_EXPORT")
const loader = new ProviderPackageLoader("bun-process", [
  {
    provider,
    service,
    target: "bun-process",
    module,
    exportName,
    loadMode: "lazy",
    importModule: async () => Object.freeze({ provider: entry }),
  },
])
const environment = Object.freeze({
  sqlite: createProviderSqlite(await loader.load(provider)),
})
const execution = createEffectExecution()
const opened = await run(openMemory(1_000), environment, execution.context)
assert(opened.kind === "success", "SQLite memory database must open")
const decodeRow: SqliteDecoder<Readonly<Record<string, unknown>>> = (row) => ({
  tag: "Decoded",
  value: row,
})

type DecoderFunction<Value> = Extract<
  SqliteDecoder<Value>,
  (row: SqliteRow) => unknown
>
type DecodeResult<Value> = ReturnType<DecoderFunction<Value>>
type Person = Readonly<{ id: number; name: string; active: boolean }>

const runDecoder = <Value>(
  value: SqliteDecoder<Value>,
  row: SqliteRow
): DecodeResult<Value> => {
  if (typeof value === "function") return value(row)
  if ("run" in value) return value.run(row)
  return value.value(row)
}

const decoder = <Value>(run: DecoderFunction<Value>): SqliteDecoder<Value> =>
  Object.freeze({ tag: "Decoder" as const, value: run })

const decoded = <Value>(value: Value): DecodeResult<Value> =>
  Object.freeze({ tag: "Decoded" as const, value }) as DecodeResult<Value>

const mapDecoder = <Input, Output>(
  transform: (value: Input) => Output,
  input: SqliteDecoder<Input>
): SqliteDecoder<Output> =>
  decoder((row) => {
    const result = runDecoder(input, row)
    return result.tag === "DecodeFailure"
      ? result
      : decoded(transform(result.value))
  })

const applyDecoder = <Input, Output>(
  wrapped: SqliteDecoder<(value: Input) => Output>,
  input: SqliteDecoder<Input>
): SqliteDecoder<Output> =>
  decoder((row) => {
    const transform = runDecoder(wrapped, row)
    if (transform.tag === "DecodeFailure") return transform
    const result = runDecoder(input, row)
    return result.tag === "DecodeFailure"
      ? result
      : decoded(transform.value(result.value))
  })

const person =
  (id: number) =>
  (name: string) =>
  (active: boolean): Person =>
    Object.freeze({ id, name, active })

const personDecoder = applyDecoder(
  applyDecoder(mapDecoder(person, int("id")), string("name")),
  bool("active")
)

const queried = await run(
  query(opened.value, { sql: "select", values: [] }, decodeRow),
  environment,
  execution.context
)
assert(queried.kind === "success", "SQLite query must succeed")
driverBytes[0] = 99
assert(
  queried.value.rows[0]?.id === 1 &&
    (queried.value.rows[0]?.payload as Uint8Array | undefined)?.[0] === 1,
  "SQLite Int and Bytes must cross as exact snapshots"
)
assertProviderConformanceCase({ id: "success", terminal: queried.kind })

const decodedPerson = await run(
  query(opened.value, { sql: "select-person", values: [] }, personDecoder),
  environment,
  execution.context
)
assert(decodedPerson.kind === "success", "SQLite person query must succeed")
assert(
  decodedPerson.value.rows[0]?.id === 1 &&
    decodedPerson.value.rows[0]?.name === "Ada" &&
    decodedPerson.value.rows[0]?.active === true,
  "SQLite applicative decoder must preserve all row fields"
)
const decodeFailed = await run(
  query(opened.value, { sql: "decode-failure", values: [] }, personDecoder),
  environment
)
assert(decodeFailed.kind === "failure", "row decode failure must stay typed")
assert(
  decodeFailed.error.tag === "RowDecodeFailure" &&
    decodeFailed.error.value.tag === "MissingColumn" &&
    decodeFailed.error.value.value === "id",
  "SQLite applicative decoder must return the leftmost row failure"
)

const busy = await run(
  query(opened.value, { sql: "busy", values: [] }, decodeRow),
  environment
)
assert(busy.kind === "failure", "SQLite busy must stay typed")
assert(
  busy.error.tag === "BusyFailure" && busy.error.value.code === "SQLITE_BUSY",
  "SQLite busy code must be preserved"
)
assertProviderConformanceCase({
  id: "typed-failure",
  terminal: "typed-failure",
})

const committed = await run(
  transaction(opened.value, transactionExecute({ sql: "insert", values: [] })),
  environment,
  execution.context
)
assert(committed.kind === "success", "SQLite transaction must commit")
failCommit = true
const commitFailed = await run(
  transaction(opened.value, transactionExecute({ sql: "insert", values: [] })),
  environment,
  execution.context
)
failCommit = false
assert(commitFailed.kind === "failure", "commit failure must stay typed")
assert(
  commitFailed.error.tag === "BusyFailure" && trace.at(-1) === "rollback",
  "failed commit must rollback the host transaction"
)
const rolledBack = await run(
  transaction(
    opened.value,
    transactionQuery({ sql: "busy", values: [] }, decodeRow)
  ),
  environment,
  execution.context
)
assert(rolledBack.kind === "failure", "typed failure must rollback")
assert(
  trace.includes("commit") && trace.includes("rollback"),
  "both transaction terminal paths must be observed"
)

const transactionExecution = createEffectExecution()
const neverCompletes: SqliteTransactionProgram<never> =
  () => (_environment, context) =>
    new Promise<never>(() => {
      trace.push("transaction-body-start")
      context?.onCancel(() => undefined)
    })
const cancelledTransaction = run(
  transaction(opened.value, neverCompletes),
  environment,
  transactionExecution.context
).catch((error: unknown) => error)
while (!trace.includes("transaction-body-start")) {
  await new Promise((resolve) => setTimeout(resolve, 0))
}
await transactionExecution.cancel()
assert(
  isEffectCancellation(await cancelledTransaction),
  "SQLite transaction cancellation must remain cancellation"
)
assert(trace.at(-1) === "rollback", "cancellation must rollback")
assertProviderConformanceCase({
  id: "cancellation",
  terminal: "cancellation",
  notifications: 0,
  lateCompletion: "discarded",
})

const defect = await run(
  query(opened.value, { sql: "invalid-row", values: [] }, decodeRow),
  environment
).catch((error: unknown) => error)
assert(
  defect instanceof ProviderBoundaryDefect && defect.stage === "result",
  "invalid SQLite row must be a result boundary defect"
)
assertProviderConformanceCase({
  id: "defect",
  terminal: "defect",
  stage: "result",
})
assertProviderConformanceCase({
  id: "invalid-value",
  boundary: "result",
  terminal: "defect",
  leakedToApplication: false,
})

await execution.cancel()
assert(trace.at(-1) === "close", "database must close after transactions")
assert(
  (await run(close(opened.value), environment)).kind === "success",
  "database close must be idempotent"
)
await loader.shutdown()
assertProviderConformanceCase({
  id: "cleanup",
  acquired: 4,
  released: 4,
  active: 0,
})
assertProviderConformanceCase({ id: "leak", activeAfterCleanup: 0 })
process.stdout.write("SQLite provider probe passed: bun-process\n")

function requiredEnvironment(name: string): string {
  const value = process.env[name]
  if (value === undefined || value.length === 0)
    throw new Error(`missing ${name}`)
  return value
}

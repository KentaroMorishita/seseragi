import {
  type EffectContext,
  registerResourceFinalizer,
  throwIfCancelled,
  type Unit,
} from "./effect"
import {
  invokeProviderOperation,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"
import type { ServiceResult } from "./service"
import {
  type Sqlite,
  type SqliteDatabase,
  type SqliteError,
  type SqliteExecuteResult,
  type SqliteFileConfig,
  type SqliteOperation,
  type SqliteRawQueryResult,
  type SqliteRow,
  type SqliteStatement,
  type SqliteTransaction,
  type SqliteValue,
  sqliteFailure,
  sqliteSuccess,
} from "./sqlite"

const named = (identity: string) =>
  Object.freeze({ kind: "named", identity } as const)
const fileConfigType = named("seseragi/sqlite::FileConfig")
const statementType = named("seseragi/sqlite::Statement")
const rowType = named("seseragi/sqlite::Row")
const queryResultType = named("seseragi/sqlite::RawQueryResult")
const executeResultType = named("seseragi/sqlite::ExecuteResult")
const errorType = named("seseragi/sqlite::Error")
const databaseType = named("seseragi/sqlite::Database")
const transactionType = named("seseragi/sqlite::Transaction")
const unit = Object.freeze({ kind: "unit" } as const)
const int = Object.freeze({ kind: "primitive", name: "int" } as const)
const record = (
  fields: ReadonlyArray<
    Readonly<{
      name: string
      type: typeof databaseType | typeof statementType | typeof transactionType
    }>
  >
) => Object.freeze({ kind: "record", fields: Object.freeze(fields) } as const)
const operation = (
  name: string,
  kind: "one-shot" | "resource",
  input: ProviderOperationContract["input"],
  success: ProviderOperationContract["success"]
): ProviderOperationContract =>
  Object.freeze({
    identity: `seseragi/sqlite::Sqlite#${name}`,
    kind,
    input,
    success,
    failure: errorType,
  })
const contracts = Object.freeze({
  openMemory: operation("openMemory", "resource", int, databaseType),
  openFile: operation("openFile", "resource", fileConfigType, databaseType),
  query: operation(
    "query",
    "one-shot",
    record([
      { name: "database", type: databaseType },
      { name: "statement", type: statementType },
    ]),
    queryResultType
  ),
  execute: operation(
    "execute",
    "one-shot",
    record([
      { name: "database", type: databaseType },
      { name: "statement", type: statementType },
    ]),
    executeResultType
  ),
  begin: operation("begin", "resource", databaseType, transactionType),
  transactionQuery: operation(
    "transactionQuery",
    "one-shot",
    record([
      { name: "transaction", type: transactionType },
      { name: "statement", type: statementType },
    ]),
    queryResultType
  ),
  transactionExecute: operation(
    "transactionExecute",
    "one-shot",
    record([
      { name: "transaction", type: transactionType },
      { name: "statement", type: statementType },
    ]),
    executeResultType
  ),
  commit: operation("commit", "one-shot", transactionType, unit),
  rollback: operation("rollback", "one-shot", transactionType, unit),
  close: operation("close", "one-shot", databaseType, unit),
})
const codecs = new ProviderCodecRegistry([
  {
    identity: fileConfigType.identity,
    encode: snapshotFileConfig,
    decode: snapshotFileConfig,
  },
  {
    identity: statementType.identity,
    encode: snapshotStatement,
    decode: snapshotStatement,
  },
  { identity: rowType.identity, encode: snapshotRow, decode: snapshotRow },
  {
    identity: queryResultType.identity,
    encode: snapshotQueryResult,
    decode: snapshotQueryResult,
  },
  {
    identity: executeResultType.identity,
    encode: snapshotExecuteResult,
    decode: snapshotExecuteResult,
  },
  {
    identity: errorType.identity,
    encode: (value) => value,
    decode: decodeError,
  },
])

type TransactionState = {
  readonly handle: SqliteTransaction
  readonly loaded: LoadedProviderEntry
  readonly parent: DatabaseState
  unregisterCleanup: () => void
  closeCompletion?: Promise<ServiceResult<SqliteError, Unit>>
}
type DatabaseState = {
  readonly handle: SqliteDatabase
  readonly loaded: LoadedProviderEntry
  readonly transactions: Set<TransactionState>
  unregisterCleanup: () => void
  closeCompletion?: Promise<ServiceResult<SqliteError, Unit>>
}

export function createProviderSqlite(loaded: LoadedProviderEntry): Sqlite {
  if (loaded.service !== "seseragi/sqlite::Sqlite") {
    throw new TypeError(
      "resolved provider does not implement seseragi/sqlite::Sqlite"
    )
  }
  const databases = new WeakMap<object, DatabaseState>()
  const transactions = new WeakMap<object, TransactionState>()
  return Object.freeze({
    async openMemory(busyTimeoutMillis: number, context: EffectContext) {
      return registerDatabase(
        loaded,
        await invoke(loaded, contracts.openMemory, busyTimeoutMillis, context),
        context,
        databases
      )
    },
    async openFile(config: SqliteFileConfig, context: EffectContext) {
      return registerDatabase(
        loaded,
        await invoke(loaded, contracts.openFile, config, context),
        context,
        databases
      )
    },
    async query(
      database: SqliteDatabase,
      statement: SqliteStatement,
      context: EffectContext
    ) {
      requireOpen(databases.get(database), "SQLite database")
      const outcome = await invoke(
        loaded,
        contracts.query,
        { database, statement },
        context
      )
      throwIfCancelled(context)
      return operationResult<SqliteRawQueryResult>(outcome)
    },
    async execute(
      database: SqliteDatabase,
      statement: SqliteStatement,
      context: EffectContext
    ) {
      requireOpen(databases.get(database), "SQLite database")
      const outcome = await invoke(
        loaded,
        contracts.execute,
        { database, statement },
        context
      )
      throwIfCancelled(context)
      return operationResult<SqliteExecuteResult>(outcome)
    },
    async begin(database: SqliteDatabase, context: EffectContext) {
      const parent = requireOpen(databases.get(database), "SQLite database")
      const outcome = await invoke(loaded, contracts.begin, database, context)
      if (outcome.kind !== "success")
        return operationResult<SqliteTransaction>(outcome)
      const handle = outcome.value as SqliteTransaction
      const state: TransactionState = {
        handle,
        loaded,
        parent,
        unregisterCleanup: () => undefined,
      }
      transactions.set(handle, state)
      parent.transactions.add(state)
      const registration = registerResourceFinalizer(context, () =>
        cleanup(closeTransactionState(state, "rollback"))
      )
      state.unregisterCleanup = registration.unregister
      await registration.ready
      throwIfCancelled(context)
      return sqliteSuccess(handle)
    },
    async transactionQuery(
      transaction: SqliteTransaction,
      statement: SqliteStatement,
      context: EffectContext
    ) {
      requireOpen(transactions.get(transaction), "SQLite transaction")
      const outcome = await invoke(
        loaded,
        contracts.transactionQuery,
        { transaction, statement },
        context
      )
      throwIfCancelled(context)
      return operationResult<SqliteRawQueryResult>(outcome)
    },
    async transactionExecute(
      transaction: SqliteTransaction,
      statement: SqliteStatement,
      context: EffectContext
    ) {
      requireOpen(transactions.get(transaction), "SQLite transaction")
      const outcome = await invoke(
        loaded,
        contracts.transactionExecute,
        { transaction, statement },
        context
      )
      throwIfCancelled(context)
      return operationResult<SqliteExecuteResult>(outcome)
    },
    async commit(transaction: SqliteTransaction, _context: EffectContext) {
      const state = transactions.get(transaction)
      if (state !== undefined) return closeTransactionState(state, "commit")
      return operationResult<Unit>(
        await invoke(loaded, contracts.commit, transaction)
      )
    },
    async rollback(transaction: SqliteTransaction, _context: EffectContext) {
      const state = transactions.get(transaction)
      if (state !== undefined) return closeTransactionState(state, "rollback")
      return operationResult<Unit>(
        await invoke(loaded, contracts.rollback, transaction)
      )
    },
    async close(database: SqliteDatabase, _context: EffectContext) {
      const state = databases.get(database)
      if (state !== undefined) return closeDatabaseState(state)
      return operationResult<Unit>(
        await invoke(loaded, contracts.close, database)
      )
    },
  })
}

async function registerDatabase(
  loaded: LoadedProviderEntry,
  outcome: ProviderBridgeOutcome,
  context: EffectContext,
  databases: WeakMap<object, DatabaseState>
): Promise<ServiceResult<SqliteError, SqliteDatabase>> {
  if (outcome.kind !== "success")
    return operationResult<SqliteDatabase>(outcome)
  const handle = outcome.value as SqliteDatabase
  const state: DatabaseState = {
    handle,
    loaded,
    transactions: new Set(),
    unregisterCleanup: () => undefined,
  }
  databases.set(handle, state)
  const registration = registerResourceFinalizer(context, () =>
    cleanup(closeDatabaseState(state))
  )
  state.unregisterCleanup = registration.unregister
  await registration.ready
  throwIfCancelled(context)
  return sqliteSuccess(handle)
}

function invoke(
  loaded: LoadedProviderEntry,
  contract: ProviderOperationContract,
  input: unknown,
  context?: EffectContext
) {
  return invokeProviderOperation({
    provider: loaded.provider,
    service: loaded.service,
    operation: contract,
    entry: loaded.entry,
    input,
    codecs,
    ...(context === undefined ? {} : { context }),
  })
}

function closeTransactionState(
  state: TransactionState,
  operation: "commit" | "rollback"
): Promise<ServiceResult<SqliteError, Unit>> {
  state.unregisterCleanup()
  state.closeCompletion ??= (async () => {
    const result = operationResult<Unit>(
      await invoke(
        state.loaded,
        operation === "commit" ? contracts.commit : contracts.rollback,
        state.handle
      )
    )
    state.parent.transactions.delete(state)
    return result
  })()
  return state.closeCompletion
}

function closeDatabaseState(
  state: DatabaseState
): Promise<ServiceResult<SqliteError, Unit>> {
  state.unregisterCleanup()
  state.closeCompletion ??= (async () => {
    let firstFailure: ServiceResult<SqliteError, never> | undefined
    for (const transaction of [...state.transactions].reverse()) {
      const result = await closeTransactionState(transaction, "rollback")
      if (result.kind === "failure" && firstFailure === undefined) {
        firstFailure = result
      }
    }
    const databaseResult = operationResult<Unit>(
      await invoke(state.loaded, contracts.close, state.handle)
    )
    return firstFailure ?? databaseResult
  })()
  return state.closeCompletion
}

function requireOpen<State extends { closeCompletion?: Promise<unknown> }>(
  state: State | undefined,
  name: string
): State {
  if (state === undefined) throw new TypeError(`${name} is not owned`)
  if (state.closeCompletion !== undefined)
    throw new TypeError(`${name} resource is closed`)
  return state
}

async function cleanup(
  completion: Promise<ServiceResult<SqliteError, Unit>>
): Promise<void> {
  const result = await completion
  if (result.kind === "failure") {
    const message =
      result.error.tag === "RowDecodeFailure"
        ? result.error.value.value
        : result.error.value.message
    throw new Error(`SQLite cleanup failed: ${message}`)
  }
}

function operationResult<Success>(
  outcome: ProviderBridgeOutcome
): ServiceResult<SqliteError, Success> {
  if (outcome.kind === "defect") throw outcome.defect
  return outcome.kind === "failure"
    ? sqliteFailure(outcome.failure as SqliteError)
    : sqliteSuccess(outcome.value as Success)
}

function snapshotFileConfig(value: unknown): SqliteFileConfig {
  const config = dataRecord(value, [
    "filename",
    "readOnly",
    "create",
    "busyTimeoutMillis",
  ])
  if (
    typeof config.filename !== "string" ||
    config.filename.length === 0 ||
    config.filename === ":memory:" ||
    typeof config.readOnly !== "boolean" ||
    typeof config.create !== "boolean" ||
    (config.readOnly && config.create) ||
    !validTimeout(config.busyTimeoutMillis)
  ) {
    throw new TypeError("SQLite file configuration is invalid")
  }
  return Object.freeze({
    filename: config.filename,
    readOnly: config.readOnly,
    create: config.create,
    busyTimeoutMillis: config.busyTimeoutMillis as number,
  })
}

function snapshotStatement(value: unknown): SqliteStatement {
  const statement = dataRecord(value, ["sql", "values"])
  if (
    typeof statement.sql !== "string" ||
    statement.sql.length === 0 ||
    !Array.isArray(statement.values)
  ) {
    throw new TypeError("SQLite statement is invalid")
  }
  return Object.freeze({
    sql: statement.sql,
    values: Object.freeze(statement.values.map(snapshotValue)),
  })
}

function snapshotRow(value: unknown): SqliteRow {
  if (!isPlainRecord(value)) throw new TypeError("SQLite row must be a record")
  const row: Record<string, SqliteValue> = {}
  for (const key of Reflect.ownKeys(value)) {
    if (typeof key !== "string")
      throw new TypeError("SQLite row key is invalid")
    const descriptor = Object.getOwnPropertyDescriptor(value, key)
    if (
      descriptor === undefined ||
      !("value" in descriptor) ||
      !descriptor.enumerable
    ) {
      throw new TypeError("SQLite row fields must be enumerable data values")
    }
    row[key] = snapshotValue(descriptor.value)
  }
  return Object.freeze(row)
}

function snapshotQueryResult(value: unknown): SqliteRawQueryResult {
  const result = dataRecord(value, ["rows"])
  if (!Array.isArray(result.rows)) {
    throw new TypeError("SQLite query result is invalid")
  }
  return Object.freeze({ rows: Object.freeze(result.rows.map(snapshotRow)) })
}

function snapshotExecuteResult(value: unknown): SqliteExecuteResult {
  const result = dataRecord(value, ["changes", "lastInsertRowId"])
  if (
    !Number.isSafeInteger(result.changes) ||
    (result.changes as number) < 0 ||
    !Number.isSafeInteger(result.lastInsertRowId)
  ) {
    throw new TypeError("SQLite execute result is invalid")
  }
  return Object.freeze({
    changes: result.changes as number,
    lastInsertRowId: result.lastInsertRowId as number,
  })
}

function snapshotValue(value: unknown): SqliteValue {
  if (
    value === null ||
    typeof value === "string" ||
    (typeof value === "number" && Number.isFinite(value))
  ) {
    return value
  }
  if (value instanceof Uint8Array) return new Uint8Array(value)
  throw new TypeError("SQLite value is outside the declared boundary")
}

function decodeError(value: unknown): SqliteError {
  const error = dataRecord(value, ["code", "message", "operation", "tag"])
  if (
    error.tag !== "QueryFailed" ||
    !isOperation(error.operation) ||
    typeof error.code !== "string" ||
    typeof error.message !== "string"
  ) {
    throw new TypeError("SQLite failure is invalid")
  }
  const driver = Object.freeze({
    operation: error.operation,
    code: error.code,
    message: error.message,
  })
  return Object.freeze({
    tag: isBusyCode(error.code) ? "BusyFailure" : "DriverFailure",
    value: driver,
  })
}

function isOperation(value: unknown): value is SqliteOperation {
  return [
    "openMemory",
    "openFile",
    "query",
    "execute",
    "begin",
    "transactionQuery",
    "transactionExecute",
    "commit",
    "rollback",
    "close",
  ].includes(value as SqliteOperation)
}

function isBusyCode(code: string): boolean {
  return (
    code === "SQLITE_BUSY" ||
    code.startsWith("SQLITE_BUSY_") ||
    code === "SQLITE_LOCKED" ||
    code.startsWith("SQLITE_LOCKED_")
  )
}

function validTimeout(value: unknown): boolean {
  return (
    Number.isSafeInteger(value) &&
    (value as number) >= 0 &&
    (value as number) <= 0x7fff_ffff
  )
}

function dataRecord(
  value: unknown,
  keys: ReadonlyArray<string>
): Record<string, unknown> {
  if (!isPlainRecord(value))
    throw new TypeError("SQLite value must be a record")
  const actual = Reflect.ownKeys(value)
  if (
    actual.length !== keys.length ||
    actual.some((key) => typeof key !== "string" || !keys.includes(key))
  ) {
    throw new TypeError("SQLite record shape is invalid")
  }
  const record: Record<string, unknown> = {}
  for (const key of keys) {
    const descriptor = Object.getOwnPropertyDescriptor(value, key)
    if (
      descriptor === undefined ||
      !("value" in descriptor) ||
      !descriptor.enumerable
    ) {
      throw new TypeError("SQLite fields must be enumerable data values")
    }
    record[key] = descriptor.value
  }
  return record
}

function isPlainRecord(value: unknown): value is Record<string, unknown> {
  return (
    typeof value === "object" &&
    value !== null &&
    [Object.prototype, null].includes(Object.getPrototypeOf(value))
  )
}

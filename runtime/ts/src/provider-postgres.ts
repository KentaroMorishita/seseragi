import {
  type EffectContext,
  registerResourceFinalizer,
  throwIfCancelled,
  type Unit,
} from "./effect"
import {
  type Postgres,
  type PostgresCursor,
  type PostgresError,
  type PostgresOperation,
  type PostgresPool,
  type PostgresConfig,
  type PostgresQuery,
  type PostgresRawQueryResult,
  type PostgresRow,
  type PostgresTransaction,
  type PostgresValue,
  postgresFailure,
  postgresSuccess,
} from "./postgres"
import {
  invokeProviderOperation,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"
import type { ServiceResult } from "./service"

const named = (identity: string) =>
  Object.freeze({ kind: "named", identity } as const)
const configType = named("seseragi/postgres::Config")
const queryType = named("seseragi/postgres::Query")
const rowType = named("seseragi/postgres::Row")
const queryResultType = named("seseragi/postgres::RawQueryResult")
const errorType = named("seseragi/postgres::Error")
const poolType = named("seseragi/postgres::Pool")
const transactionType = named("seseragi/postgres::Transaction")
const cursorType = named("seseragi/postgres::Cursor")
const unit = Object.freeze({ kind: "unit" } as const)
const int = Object.freeze({ kind: "primitive", name: "int" } as const)
const rowsType = Object.freeze({ kind: "array", items: rowType } as const)
const record = (
  fields: ReadonlyArray<
    Readonly<{
      name: string
      type:
        | typeof poolType
        | typeof queryType
        | typeof transactionType
        | typeof cursorType
        | typeof int
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
    identity: `seseragi/postgres::Postgres#${name}`,
    kind,
    input,
    success,
    failure: errorType,
  })
const contracts = Object.freeze({
  openPool: operation("openPool", "resource", configType, poolType),
  query: operation(
    "query",
    "one-shot",
    record([
      { name: "pool", type: poolType },
      { name: "query", type: queryType },
    ]),
    queryResultType
  ),
  begin: operation("begin", "resource", poolType, transactionType),
  transactionQuery: operation(
    "transactionQuery",
    "one-shot",
    record([
      { name: "transaction", type: transactionType },
      { name: "query", type: queryType },
    ]),
    queryResultType
  ),
  commit: operation("commit", "one-shot", transactionType, unit),
  rollback: operation("rollback", "one-shot", transactionType, unit),
  openCursor: operation(
    "openCursor",
    "resource",
    record([
      { name: "pool", type: poolType },
      { name: "query", type: queryType },
    ]),
    cursorType
  ),
  fetch: operation(
    "fetch",
    "one-shot",
    record([
      { name: "cursor", type: cursorType },
      { name: "limit", type: int },
    ]),
    rowsType
  ),
  closeCursor: operation("closeCursor", "one-shot", cursorType, unit),
  closePool: operation("closePool", "one-shot", poolType, unit),
})
const codecs = new ProviderCodecRegistry([
  {
    identity: configType.identity,
    encode: snapshotConfig,
    decode: snapshotConfig,
  },
  {
    identity: queryType.identity,
    encode: snapshotQuery,
    decode: snapshotQuery,
  },
  { identity: rowType.identity, encode: snapshotRow, decode: snapshotRow },
  {
    identity: queryResultType.identity,
    encode: snapshotQueryResult,
    decode: snapshotQueryResult,
  },
  {
    identity: errorType.identity,
    encode: (value) => value,
    decode: decodeError,
  },
])

type CursorState = {
  readonly handle: PostgresCursor
  readonly loaded: LoadedProviderEntry
  unregisterCleanup: () => void
  closeCompletion?: Promise<ServiceResult<PostgresError, Unit>>
}
type TransactionState = {
  readonly handle: PostgresTransaction
  readonly loaded: LoadedProviderEntry
  readonly parent: PoolState
  unregisterCleanup: () => void
  commitCompletion?: Promise<ServiceResult<PostgresError, Unit>>
  rollbackCompletion?: Promise<ServiceResult<PostgresError, Unit>>
  closeCompletion?: Promise<ServiceResult<PostgresError, Unit>>
}
type PoolState = {
  readonly handle: PostgresPool
  readonly loaded: LoadedProviderEntry
  readonly cursors: Set<CursorState>
  readonly transactions: Set<TransactionState>
  unregisterCleanup: () => void
  closeCompletion?: Promise<ServiceResult<PostgresError, Unit>>
}

export function createProviderPostgres(loaded: LoadedProviderEntry): Postgres {
  if (loaded.service !== "seseragi/postgres::Postgres") {
    throw new TypeError(
      "resolved provider does not implement seseragi/postgres::Postgres"
    )
  }
  const pools = new WeakMap<object, PoolState>()
  const cursors = new WeakMap<object, CursorState>()
  const transactions = new WeakMap<object, TransactionState>()
  return Object.freeze({
    async openPool(options: PostgresConfig, context: EffectContext) {
      const outcome = await invoke(loaded, contracts.openPool, options, context)
      if (outcome.kind !== "success")
        return operationResult<PostgresPool>(outcome)
      const handle = outcome.value as PostgresPool
      const state: PoolState = {
        handle,
        loaded,
        cursors: new Set(),
        transactions: new Set(),
        unregisterCleanup: () => undefined,
      }
      pools.set(handle, state)
      const registration = registerResourceFinalizer(context, () =>
        cleanup(closePoolState(state))
      )
      state.unregisterCleanup = registration.unregister
      await registration.ready
      throwIfCancelled(context)
      return postgresSuccess(handle)
    },
    async query(
      pool: PostgresPool,
      query: PostgresQuery,
      context: EffectContext
    ) {
      ensureOpen(pools.get(pool), "PostgreSQL pool")
      const outcome = await invoke(
        loaded,
        contracts.query,
        { pool, query },
        context
      )
      throwIfCancelled(context)
      return operationResult<PostgresRawQueryResult>(outcome)
    },
    async begin(pool: PostgresPool, context: EffectContext) {
      const poolState = ensureOpen(pools.get(pool), "PostgreSQL pool")
      const outcome = await invoke(loaded, contracts.begin, pool, context)
      if (outcome.kind !== "success")
        return operationResult<PostgresTransaction>(outcome)
      const handle = outcome.value as PostgresTransaction
      const state: TransactionState = {
        handle,
        loaded,
        parent: poolState,
        unregisterCleanup: () => undefined,
      }
      transactions.set(handle, state)
      poolState.transactions.add(state)
      const registration = registerResourceFinalizer(context, () =>
        cleanup(closeTransactionState(state, "rollback"))
      )
      state.unregisterCleanup = registration.unregister
      await registration.ready
      throwIfCancelled(context)
      return postgresSuccess(handle)
    },
    async transactionQuery(
      transaction: PostgresTransaction,
      query: PostgresQuery,
      context: EffectContext
    ) {
      ensureOpen(transactions.get(transaction), "PostgreSQL transaction")
      const outcome = await invoke(
        loaded,
        contracts.transactionQuery,
        { transaction, query },
        context
      )
      throwIfCancelled(context)
      return operationResult<PostgresRawQueryResult>(outcome)
    },
    async commit(transaction: PostgresTransaction, _context: EffectContext) {
      const state = transactions.get(transaction)
      if (state !== undefined) return closeTransactionState(state, "commit")
      return operationResult<Unit>(
        await invoke(loaded, contracts.commit, transaction)
      )
    },
    async rollback(transaction: PostgresTransaction, _context: EffectContext) {
      const state = transactions.get(transaction)
      if (state !== undefined) return closeTransactionState(state, "rollback")
      return operationResult<Unit>(
        await invoke(loaded, contracts.rollback, transaction)
      )
    },
    async openCursor(
      pool: PostgresPool,
      query: PostgresQuery,
      context: EffectContext
    ) {
      const poolState = pools.get(pool)
      ensureOpen(poolState, "PostgreSQL pool")
      const outcome = await invoke(
        loaded,
        contracts.openCursor,
        { pool, query },
        context
      )
      if (outcome.kind !== "success")
        return operationResult<PostgresCursor>(outcome)
      const handle = outcome.value as PostgresCursor
      const state: CursorState = {
        handle,
        loaded,
        unregisterCleanup: () => undefined,
      }
      cursors.set(handle, state)
      poolState?.cursors.add(state)
      const registration = registerResourceFinalizer(context, () =>
        cleanup(closeCursorState(state))
      )
      state.unregisterCleanup = registration.unregister
      await registration.ready
      throwIfCancelled(context)
      return postgresSuccess(handle)
    },
    async fetch(cursor: PostgresCursor, limit: number, context: EffectContext) {
      ensureOpen(cursors.get(cursor), "PostgreSQL cursor")
      const outcome = await invoke(
        loaded,
        contracts.fetch,
        { cursor, limit },
        context
      )
      throwIfCancelled(context)
      return operationResult<ReadonlyArray<PostgresRow>>(outcome)
    },
    async closeCursor(cursor: PostgresCursor, _context: EffectContext) {
      const state = cursors.get(cursor)
      if (state !== undefined) return closeCursorState(state)
      return operationResult<Unit>(
        await invoke(loaded, contracts.closeCursor, cursor)
      )
    },
    async closePool(pool: PostgresPool, _context: EffectContext) {
      const state = pools.get(pool)
      if (state !== undefined) return closePoolState(state)
      return operationResult<Unit>(
        await invoke(loaded, contracts.closePool, pool)
      )
    },
  })
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

function closeCursorState(
  state: CursorState
): Promise<ServiceResult<PostgresError, Unit>> {
  state.unregisterCleanup()
  state.closeCompletion ??= (async () =>
    operationResult<Unit>(
      await invoke(state.loaded, contracts.closeCursor, state.handle)
    ))()
  return state.closeCompletion
}

function closeTransactionState(
  state: TransactionState,
  operation: "commit" | "rollback"
): Promise<ServiceResult<PostgresError, Unit>> {
  if (operation === "commit") {
    if (state.rollbackCompletion !== undefined) return state.rollbackCompletion
    state.commitCompletion ??= runTransactionOperation(state, "commit")
    state.closeCompletion ??= state.commitCompletion
    return state.commitCompletion
  }
  state.rollbackCompletion ??= rollbackTransactionState(state)
  state.closeCompletion ??= state.rollbackCompletion
  return state.rollbackCompletion
}

async function rollbackTransactionState(
  state: TransactionState
): Promise<ServiceResult<PostgresError, Unit>> {
  if (state.commitCompletion !== undefined) {
    try {
      const committed = await state.commitCompletion
      if (committed.kind === "success") return committed
    } catch {
      // Cleanup still owns rollback after a defective commit.
    }
  }
  return runTransactionOperation(state, "rollback")
}

async function runTransactionOperation(
  state: TransactionState,
  operation: "commit" | "rollback"
): Promise<ServiceResult<PostgresError, Unit>> {
  const result = operationResult<Unit>(
    await invoke(
      state.loaded,
      operation === "commit" ? contracts.commit : contracts.rollback,
      state.handle
    )
  )
  if (result.kind === "success") {
    state.unregisterCleanup()
    state.parent.transactions.delete(state)
  }
  return result
}

function closePoolState(
  state: PoolState
): Promise<ServiceResult<PostgresError, Unit>> {
  state.unregisterCleanup()
  state.closeCompletion ??= (async () => {
    let firstFailure: ServiceResult<PostgresError, never> | undefined
    for (const transaction of [...state.transactions].reverse()) {
      const result = await closeTransactionState(transaction, "rollback")
      if (result.kind === "failure" && firstFailure === undefined) {
        firstFailure = result
      }
    }
    for (const cursor of [...state.cursors].reverse()) {
      const result = await closeCursorState(cursor)
      if (result.kind === "failure" && firstFailure === undefined) {
        firstFailure = result
      }
    }
    const poolResult = operationResult<Unit>(
      await invoke(state.loaded, contracts.closePool, state.handle)
    )
    return firstFailure ?? poolResult
  })()
  return state.closeCompletion
}

function ensureOpen<State extends { closeCompletion?: Promise<unknown> }>(
  state: State | undefined,
  name: string
): State {
  if (state === undefined) throw new TypeError(`${name} is not owned`)
  if (state.closeCompletion !== undefined) {
    throw new TypeError(`${name} resource is closed`)
  }
  return state
}

async function cleanup(
  completion: Promise<ServiceResult<PostgresError, Unit>>
): Promise<void> {
  const result = await completion
  if (result.kind === "failure") {
    const message =
      result.error.tag === "DriverFailure"
        ? result.error.value.message
        : result.error.value.value
    throw new Error(`PostgreSQL cleanup failed: ${message}`)
  }
}

function operationResult<Success>(
  outcome: ProviderBridgeOutcome
): ServiceResult<PostgresError, Success> {
  if (outcome.kind === "defect") throw outcome.defect
  return outcome.kind === "failure"
    ? postgresFailure(outcome.failure as PostgresError)
    : postgresSuccess(outcome.value as Success)
}

function snapshotConfig(value: unknown): PostgresConfig {
  const options = dataRecord(value, ["connectionString", "maxConnections"])
  if (
    typeof options.connectionString !== "string" ||
    options.connectionString.length === 0 ||
    !Number.isSafeInteger(options.maxConnections) ||
    (options.maxConnections as number) <= 0 ||
    (options.maxConnections as number) > 0x7fff_ffff
  ) {
    throw new TypeError("PostgreSQL pool configuration is invalid")
  }
  return Object.freeze({
    connectionString: options.connectionString,
    maxConnections: options.maxConnections as number,
  })
}

function snapshotQuery(value: unknown): PostgresQuery {
  const query = dataRecord(value, ["text", "values"])
  if (
    typeof query.text !== "string" ||
    query.text.length === 0 ||
    !Array.isArray(query.values)
  ) {
    throw new TypeError("PostgreSQL query is invalid")
  }
  return Object.freeze({
    text: query.text,
    values: Object.freeze(query.values.map(snapshotValue)),
  })
}

function snapshotRow(value: unknown): PostgresRow {
  if (!isPlainRecord(value))
    throw new TypeError("PostgreSQL row must be a record")
  const row: Record<string, PostgresValue> = {}
  for (const key of Reflect.ownKeys(value)) {
    if (typeof key !== "string")
      throw new TypeError("PostgreSQL row key is invalid")
    const descriptor = Object.getOwnPropertyDescriptor(value, key)
    if (
      descriptor === undefined ||
      !("value" in descriptor) ||
      !descriptor.enumerable
    ) {
      throw new TypeError(
        "PostgreSQL row fields must be enumerable data values"
      )
    }
    row[key] = snapshotValue(descriptor.value)
  }
  return Object.freeze(row)
}

function snapshotQueryResult(value: unknown): PostgresRawQueryResult {
  const result = dataRecord(value, ["command", "rowCount", "rows"])
  if (
    typeof result.command !== "string" ||
    !Number.isSafeInteger(result.rowCount) ||
    (result.rowCount as number) < 0 ||
    !Array.isArray(result.rows)
  ) {
    throw new TypeError("PostgreSQL query result is invalid")
  }
  return Object.freeze({
    rows: Object.freeze(result.rows.map(snapshotRow)),
    rowCount: result.rowCount as number,
    command: result.command,
  })
}

function snapshotValue(value: unknown): PostgresValue {
  if (
    value === null ||
    typeof value === "string" ||
    typeof value === "boolean" ||
    (typeof value === "number" && Number.isFinite(value))
  ) {
    return value
  }
  if (value instanceof Uint8Array) return new Uint8Array(value)
  throw new TypeError("PostgreSQL value is outside the declared boundary")
}

function decodeError(value: unknown): PostgresError {
  const error = dataRecord(value, ["code", "message", "operation", "tag"])
  if (
    error.tag !== "QueryFailed" ||
    !isOperation(error.operation) ||
    typeof error.code !== "string" ||
    typeof error.message !== "string"
  ) {
    throw new TypeError("PostgreSQL failure is invalid")
  }
  return Object.freeze({
    tag: "DriverFailure",
    value: Object.freeze({
      operation: error.operation,
      code: error.code,
      message: error.message,
    }),
  })
}

function isOperation(value: unknown): value is PostgresOperation {
  return [
    "openPool",
    "query",
    "begin",
    "transactionQuery",
    "commit",
    "rollback",
    "openCursor",
    "fetch",
    "closeCursor",
    "closePool",
  ].includes(value as PostgresOperation)
}

function dataRecord(
  value: unknown,
  keys: ReadonlyArray<string>
): Record<string, unknown> {
  if (!isPlainRecord(value))
    throw new TypeError("PostgreSQL value must be a record")
  const actual = Reflect.ownKeys(value)
  if (
    actual.length !== keys.length ||
    actual.some((key) => typeof key !== "string" || !keys.includes(key))
  ) {
    throw new TypeError("PostgreSQL record shape is invalid")
  }
  const record: Record<string, unknown> = {}
  for (const key of keys) {
    const descriptor = Object.getOwnPropertyDescriptor(value, key)
    if (
      descriptor === undefined ||
      !("value" in descriptor) ||
      !descriptor.enumerable
    ) {
      throw new TypeError("PostgreSQL fields must be enumerable data values")
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

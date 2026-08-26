import type { Bytes } from "./bytes"
import {
  createEffectExecution,
  type Effect,
  type EffectContext,
  fail,
  run,
  type Unit,
} from "./effect"
import type { ProviderHandle } from "./provider"
import {
  type ServiceResult,
  serviceEffect,
  serviceFailure,
  serviceSuccess,
} from "./service"

export type SqliteValue = string | number | null | Uint8Array
export type SqliteRow = Readonly<Record<string, SqliteValue>>
export type SqliteFileConfig = Readonly<{
  filename: string
  readOnly: boolean
  create: boolean
  busyTimeoutMillis: number
}>
export type SqliteStatement = Readonly<{
  sql: string
  values: ReadonlyArray<SqliteValue>
}>
export type SqliteRawQueryResult = Readonly<{
  rows: ReadonlyArray<SqliteRow>
}>
export type SqliteExecuteResult = Readonly<{
  changes: number
  lastInsertRowId: number
}>
export type SqliteOperation =
  | "openMemory"
  | "openFile"
  | "query"
  | "execute"
  | "begin"
  | "transactionQuery"
  | "transactionExecute"
  | "commit"
  | "rollback"
  | "close"
export type SqliteDriverError = Readonly<{
  operation: SqliteOperation
  code: string
  message: string
}>
export type SqliteRowDecodeError =
  | Readonly<{ tag: "MissingColumn"; value: string }>
  | Readonly<{ tag: "UnexpectedColumnType"; value: string }>
  | Readonly<{ tag: "IntOutsideRange"; value: string }>
export type SqliteError =
  | Readonly<{ tag: "BusyFailure"; value: SqliteDriverError }>
  | Readonly<{ tag: "DriverFailure"; value: SqliteDriverError }>
  | Readonly<{ tag: "RowDecodeFailure"; value: SqliteRowDecodeError }>
export type SqliteDatabase = ProviderHandle
export type SqliteTransaction = ProviderHandle
export type SqliteQueryResult<Value> = Readonly<{
  rows: ReadonlyArray<Value>
}>

export type Sqlite = Readonly<{
  openMemory: (
    busyTimeoutMillis: number,
    context: EffectContext
  ) => Promise<ServiceResult<SqliteError, SqliteDatabase>>
  openFile: (
    config: SqliteFileConfig,
    context: EffectContext
  ) => Promise<ServiceResult<SqliteError, SqliteDatabase>>
  query: (
    database: SqliteDatabase,
    statement: SqliteStatement,
    context: EffectContext
  ) => Promise<ServiceResult<SqliteError, SqliteRawQueryResult>>
  execute: (
    database: SqliteDatabase,
    statement: SqliteStatement,
    context: EffectContext
  ) => Promise<ServiceResult<SqliteError, SqliteExecuteResult>>
  begin: (
    database: SqliteDatabase,
    context: EffectContext
  ) => Promise<ServiceResult<SqliteError, SqliteTransaction>>
  transactionQuery: (
    transaction: SqliteTransaction,
    statement: SqliteStatement,
    context: EffectContext
  ) => Promise<ServiceResult<SqliteError, SqliteRawQueryResult>>
  transactionExecute: (
    transaction: SqliteTransaction,
    statement: SqliteStatement,
    context: EffectContext
  ) => Promise<ServiceResult<SqliteError, SqliteExecuteResult>>
  commit: (
    transaction: SqliteTransaction,
    context: EffectContext
  ) => Promise<ServiceResult<SqliteError, Unit>>
  rollback: (
    transaction: SqliteTransaction,
    context: EffectContext
  ) => Promise<ServiceResult<SqliteError, Unit>>
  close: (
    database: SqliteDatabase,
    context: EffectContext
  ) => Promise<ServiceResult<SqliteError, Unit>>
}>
export type SqliteEnvironment = Readonly<{ sqlite: Sqlite }>

type DecodeResult<Value> =
  | Readonly<{ tag: "DecodeFailure"; value: SqliteRowDecodeError }>
  | Readonly<{ tag: "Decoded"; value: Value }>
type SqliteDecoderFunction<Value> = (row: SqliteRow) => DecodeResult<Value>
export type SqliteDecoder<Value> =
  | SqliteDecoderFunction<Value>
  | Readonly<{ run: SqliteDecoderFunction<Value> }>
  | Readonly<{ tag: "Decoder"; value: SqliteDecoderFunction<Value> }>
export type SqliteTransactionProgram<Value> = (
  transaction: SqliteTransaction
) => Effect<SqliteEnvironment, SqliteError, Value>

export const textValue = (value: string): SqliteValue => value
export const intValue = (value: number): SqliteValue => value
export const floatValue = (value: number): SqliteValue => value
export const boolValue = (value: boolean): SqliteValue => (value ? 1 : 0)
export const bytesValue = (value: Bytes): SqliteValue => new Uint8Array(value)
export const nullValue = (_unit: Unit): SqliteValue => null
export const emptyValues = (_unit: Unit): ReadonlyArray<SqliteValue> =>
  Object.freeze([])

export const string = (column: string): SqliteDecoder<string> =>
  decodeColumn(column, (value) =>
    typeof value === "string" ? decoded(value) : unexpected(column)
  )

export const int = (column: string): SqliteDecoder<number> =>
  decodeColumn(column, (value) => {
    if (typeof value !== "number") return unexpected(column)
    return Number.isSafeInteger(value)
      ? decoded(value)
      : decodeFailure({ tag: "IntOutsideRange", value: column })
  })

export const float = (column: string): SqliteDecoder<number> =>
  decodeColumn(column, (value) =>
    typeof value === "number" && Number.isFinite(value)
      ? decoded(value)
      : unexpected(column)
  )

export const bool = (column: string): SqliteDecoder<boolean> =>
  decodeColumn(column, (value) =>
    value === 0
      ? decoded(false)
      : value === 1
        ? decoded(true)
        : unexpected(column)
  )

export const bytes = (column: string): SqliteDecoder<Bytes> =>
  decodeColumn(column, (value) =>
    value instanceof Uint8Array
      ? decoded(new Uint8Array(value) as Bytes)
      : unexpected(column)
  )

export function openMemory(
  busyTimeoutMillis: number
): Effect<SqliteEnvironment, SqliteError, SqliteDatabase> {
  return serviceEffect((environment, context) =>
    environment.sqlite.openMemory(busyTimeoutMillis, context)
  )
}

export function openFile(
  config: SqliteFileConfig
): Effect<SqliteEnvironment, SqliteError, SqliteDatabase> {
  return serviceEffect((environment, context) =>
    environment.sqlite.openFile(config, context)
  )
}

export function query<Value>(
  database: SqliteDatabase,
  statement: SqliteStatement,
  decoder: SqliteDecoder<Value>
): Effect<SqliteEnvironment, SqliteError, SqliteQueryResult<Value>> {
  return decodeQueryResult(
    serviceEffect((environment, context) =>
      environment.sqlite.query(database, statement, context)
    ),
    decoder
  )
}

export function execute(
  database: SqliteDatabase,
  statement: SqliteStatement
): Effect<SqliteEnvironment, SqliteError, SqliteExecuteResult> {
  return serviceEffect((environment, context) =>
    environment.sqlite.execute(database, statement, context)
  )
}

export const transactionQuery =
  <Value>(
    statement: SqliteStatement,
    decoder: SqliteDecoder<Value>
  ): SqliteTransactionProgram<SqliteQueryResult<Value>> =>
  (
    transaction: SqliteTransaction
  ): Effect<SqliteEnvironment, SqliteError, SqliteQueryResult<Value>> =>
    decodeQueryResult(
      serviceEffect((environment, context) =>
        environment.sqlite.transactionQuery(transaction, statement, context)
      ),
      decoder
    )

export const transactionExecute =
  (statement: SqliteStatement): SqliteTransactionProgram<SqliteExecuteResult> =>
  (
    transaction: SqliteTransaction
  ): Effect<SqliteEnvironment, SqliteError, SqliteExecuteResult> =>
    serviceEffect((environment, context) =>
      environment.sqlite.transactionExecute(transaction, statement, context)
    )

export const transactionThen =
  <First, Second>(
    first: SqliteTransactionProgram<First>,
    second: SqliteTransactionProgram<Second>
  ): SqliteTransactionProgram<Second> =>
  (transaction) =>
  async (environment, context) => {
    const left = await run(first(transaction), environment, context)
    if (left.kind === "failure") return fail(left.error)(environment, context)
    const right = await run(second(transaction), environment, context)
    return right.kind === "failure"
      ? fail(right.error)(environment, context)
      : right.value
  }

export function transaction<Value>(
  database: SqliteDatabase,
  program: SqliteTransactionProgram<Value>
): Effect<SqliteEnvironment, SqliteError, Value> {
  return async (environment, context) => {
    const execution =
      context === undefined ? createEffectExecution() : undefined
    try {
      const active = context ?? execution?.context
      if (active === undefined)
        throw new TypeError("SQLite transaction context is unavailable")
      const begun = await run(beginSqlite(database), environment, active)
      if (begun.kind === "failure")
        return fail(begun.error)(environment, active)
      let completed: ServiceResult<SqliteError, Value>
      try {
        const used = await run(program(begun.value), environment, active)
        if (used.kind === "failure") {
          const rolledBack = await run(
            rollbackSqlite(begun.value),
            environment,
            active
          )
          completed = sqliteFailure(
            rolledBack.kind === "failure" ? rolledBack.error : used.error
          )
        } else {
          const committed = await run(
            commitSqlite(begun.value),
            environment,
            active
          )
          completed =
            committed.kind === "failure"
              ? sqliteFailure(committed.error)
              : sqliteSuccess(used.value)
        }
      } catch (cause) {
        await rollbackAfterInterruption(begun.value, environment)
        throw cause
      }
      return completed.kind === "failure"
        ? fail(completed.error)(environment, active)
        : completed.value
    } finally {
      await execution?.close()
    }
  }
}

export function close(
  database: SqliteDatabase
): Effect<SqliteEnvironment, SqliteError, Unit> {
  return serviceEffect((environment, context) =>
    environment.sqlite.close(database, context)
  )
}

export function sqliteSuccess<Success>(
  value: Success
): ServiceResult<never, Success> {
  return serviceSuccess(value)
}

export function sqliteFailure(
  error: SqliteError
): ServiceResult<SqliteError, never> {
  return serviceFailure(error)
}

function beginSqlite(
  database: SqliteDatabase
): Effect<SqliteEnvironment, SqliteError, SqliteTransaction> {
  return serviceEffect((environment, context) =>
    environment.sqlite.begin(database, context)
  )
}

function commitSqlite(
  transaction: SqliteTransaction
): Effect<SqliteEnvironment, SqliteError, Unit> {
  return serviceEffect((environment, context) =>
    environment.sqlite.commit(transaction, context)
  )
}

function rollbackSqlite(
  transaction: SqliteTransaction
): Effect<SqliteEnvironment, SqliteError, Unit> {
  return serviceEffect((environment, context) =>
    environment.sqlite.rollback(transaction, context)
  )
}

function decodeQueryResult<Value>(
  effect: Effect<SqliteEnvironment, SqliteError, SqliteRawQueryResult>,
  decoder: SqliteDecoder<Value>
): Effect<SqliteEnvironment, SqliteError, SqliteQueryResult<Value>> {
  return async (environment, context) => {
    const result = await run(effect, environment, context)
    if (result.kind === "failure")
      return fail(result.error)(environment, context)
    const rows = decodeRows(result.value.rows, decoder)
    if (rows.tag === "DecodeFailure")
      return fail(rowDecodeFailure(rows.value))(environment, context)
    return Object.freeze({ rows: rows.value })
  }
}

function decodeRows<Value>(
  rows: ReadonlyArray<SqliteRow>,
  decoder: SqliteDecoder<Value>
): DecodeResult<ReadonlyArray<Value>> {
  const values: Value[] = []
  for (const row of rows) {
    const result = runDecoder(decoder, row)
    if (result.tag === "DecodeFailure") return result
    values.push(result.value)
  }
  return decoded(Object.freeze(values))
}

function decodeColumn<Value>(
  column: string,
  decode: (value: SqliteValue) => DecodeResult<Value>
): SqliteDecoder<Value> {
  return decoder((row) =>
    Object.hasOwn(row, column)
      ? decode(row[column] as SqliteValue)
      : decodeFailure({ tag: "MissingColumn", value: column })
  )
}

function decoder<Value>(
  run: SqliteDecoderFunction<Value>
): SqliteDecoder<Value> {
  return Object.freeze({ tag: "Decoder" as const, value: run })
}

function runDecoder<Value>(
  decoder: SqliteDecoder<Value>,
  row: SqliteRow
): DecodeResult<Value> {
  if (typeof decoder === "function") return decoder(row)
  if ("run" in decoder) return decoder.run(row)
  return decoder.value(row)
}

const decoded = <Value>(value: Value): DecodeResult<Value> =>
  Object.freeze({ tag: "Decoded", value })

const decodeFailure = (value: SqliteRowDecodeError): DecodeResult<never> =>
  Object.freeze({ tag: "DecodeFailure", value })

const unexpected = (column: string): DecodeResult<never> =>
  decodeFailure({ tag: "UnexpectedColumnType", value: column })

const rowDecodeFailure = (value: SqliteRowDecodeError): SqliteError =>
  Object.freeze({ tag: "RowDecodeFailure", value })

async function rollbackAfterInterruption(
  transaction: SqliteTransaction,
  environment: SqliteEnvironment
): Promise<void> {
  const execution = createEffectExecution()
  try {
    const result = await run(
      rollbackSqlite(transaction),
      environment,
      execution.context
    )
    if (result.kind === "failure") {
      const failure = result.error
      const message =
        failure.tag === "RowDecodeFailure"
          ? failure.value.value
          : failure.value.message
      throw new Error(`SQLite rollback failed: ${message}`)
    }
  } finally {
    await execution.close()
  }
}

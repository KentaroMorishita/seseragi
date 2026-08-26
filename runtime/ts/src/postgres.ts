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

export type PostgresValue = string | number | boolean | null | Uint8Array
export type PostgresRow = Readonly<Record<string, PostgresValue>>
export type PostgresConfig = Readonly<{
  connectionString: string
  maxConnections: number
}>
export type PostgresQuery = Readonly<{
  text: string
  values: ReadonlyArray<PostgresValue>
}>
export type PostgresRawQueryResult = Readonly<{
  rows: ReadonlyArray<PostgresRow>
  rowCount: number
  command: string
}>
export type PostgresOperation =
  | "openPool"
  | "query"
  | "begin"
  | "transactionQuery"
  | "commit"
  | "rollback"
  | "openCursor"
  | "fetch"
  | "closeCursor"
  | "closePool"
export type PostgresDriverError = Readonly<{
  operation: PostgresOperation
  code: string
  message: string
}>
export type PostgresRowDecodeError =
  | Readonly<{ tag: "MissingColumn"; value: string }>
  | Readonly<{ tag: "UnexpectedColumnType"; value: string }>
  | Readonly<{ tag: "IntOutsideRange"; value: string }>
export type PostgresError =
  | Readonly<{ tag: "DriverFailure"; value: PostgresDriverError }>
  | Readonly<{ tag: "RowDecodeFailure"; value: PostgresRowDecodeError }>
export type PostgresPool = ProviderHandle
export type PostgresTransaction = ProviderHandle
export type PostgresCursor = ProviderHandle
export type PostgresQueryResult<Value> = Readonly<{
  rows: ReadonlyArray<Value>
  rowCount: number
  command: string
}>

export type Postgres = Readonly<{
  openPool: (
    config: PostgresConfig,
    context: EffectContext
  ) => Promise<ServiceResult<PostgresError, PostgresPool>>
  query: (
    pool: PostgresPool,
    query: PostgresQuery,
    context: EffectContext
  ) => Promise<ServiceResult<PostgresError, PostgresRawQueryResult>>
  begin: (
    pool: PostgresPool,
    context: EffectContext
  ) => Promise<ServiceResult<PostgresError, PostgresTransaction>>
  transactionQuery: (
    transaction: PostgresTransaction,
    query: PostgresQuery,
    context: EffectContext
  ) => Promise<ServiceResult<PostgresError, PostgresRawQueryResult>>
  commit: (
    transaction: PostgresTransaction,
    context: EffectContext
  ) => Promise<ServiceResult<PostgresError, Unit>>
  rollback: (
    transaction: PostgresTransaction,
    context: EffectContext
  ) => Promise<ServiceResult<PostgresError, Unit>>
  openCursor: (
    pool: PostgresPool,
    query: PostgresQuery,
    context: EffectContext
  ) => Promise<ServiceResult<PostgresError, PostgresCursor>>
  fetch: (
    cursor: PostgresCursor,
    limit: number,
    context: EffectContext
  ) => Promise<ServiceResult<PostgresError, ReadonlyArray<PostgresRow>>>
  closeCursor: (
    cursor: PostgresCursor,
    context: EffectContext
  ) => Promise<ServiceResult<PostgresError, Unit>>
  closePool: (
    pool: PostgresPool,
    context: EffectContext
  ) => Promise<ServiceResult<PostgresError, Unit>>
}>
export type PostgresEnvironment = Readonly<{ postgres: Postgres }>

type DecodeResult<Value> =
  | Readonly<{ tag: "DecodeFailure"; value: PostgresRowDecodeError }>
  | Readonly<{ tag: "Decoded"; value: Value }>
type PostgresDecoderFunction<Value> = (row: PostgresRow) => DecodeResult<Value>
export type PostgresDecoder<Value> =
  | PostgresDecoderFunction<Value>
  | Readonly<{ run: PostgresDecoderFunction<Value> }>
  | Readonly<{ tag: "Decoder"; value: PostgresDecoderFunction<Value> }>
export type PostgresTransactionProgram<Value> = (
  transaction: PostgresTransaction
) => Effect<PostgresEnvironment, PostgresError, Value>

export const textValue = (value: string): PostgresValue => value
export const intValue = (value: number): PostgresValue => value
export const floatValue = (value: number): PostgresValue => value
export const boolValue = (value: boolean): PostgresValue => value
export const bytesValue = (value: Bytes): PostgresValue => new Uint8Array(value)
export const nullValue = (_unit: Unit): PostgresValue => null
export const emptyValues = (_unit: Unit): ReadonlyArray<PostgresValue> =>
  Object.freeze([])

export const string = (column: string): PostgresDecoder<string> =>
  decodeColumn(column, (value) =>
    typeof value === "string" ? decoded(value) : unexpected(column)
  )

export const int = (column: string): PostgresDecoder<number> =>
  decodeColumn(column, (value) => {
    if (typeof value !== "number") return unexpected(column)
    return Number.isSafeInteger(value)
      ? decoded(value)
      : decodeFailure({ tag: "IntOutsideRange", value: column })
  })

export const float = (column: string): PostgresDecoder<number> =>
  decodeColumn(column, (value) =>
    typeof value === "number" && Number.isFinite(value)
      ? decoded(value)
      : unexpected(column)
  )

export const bool = (column: string): PostgresDecoder<boolean> =>
  decodeColumn(column, (value) =>
    typeof value === "boolean" ? decoded(value) : unexpected(column)
  )

export const bytes = (column: string): PostgresDecoder<Bytes> =>
  decodeColumn(column, (value) =>
    value instanceof Uint8Array
      ? decoded(new Uint8Array(value) as Bytes)
      : unexpected(column)
  )

export function openPool(
  config: PostgresConfig
): Effect<PostgresEnvironment, PostgresError, PostgresPool> {
  return serviceEffect((environment, context) =>
    environment.postgres.openPool(config, context)
  )
}

export function query<Value>(
  pool: PostgresPool,
  input: PostgresQuery,
  decoder: PostgresDecoder<Value>
): Effect<PostgresEnvironment, PostgresError, PostgresQueryResult<Value>> {
  return decodeQueryResult(
    serviceEffect((environment, context) =>
      environment.postgres.query(pool, input, context)
    ),
    decoder
  )
}

export const transactionQuery =
  <Value>(
    input: PostgresQuery,
    decoder: PostgresDecoder<Value>
  ): PostgresTransactionProgram<PostgresQueryResult<Value>> =>
  (
    transaction: PostgresTransaction
  ): Effect<PostgresEnvironment, PostgresError, PostgresQueryResult<Value>> =>
    decodeQueryResult(
      serviceEffect((environment, context) =>
        environment.postgres.transactionQuery(transaction, input, context)
      ),
      decoder
    )

export function transaction<Value>(
  pool: PostgresPool,
  program: PostgresTransactionProgram<Value>
): Effect<PostgresEnvironment, PostgresError, Value> {
  return async (environment, context) => {
    const execution =
      context === undefined ? createEffectExecution() : undefined
    try {
      const active = context ?? execution?.context
      if (active === undefined)
        throw new TypeError("PostgreSQL transaction context is unavailable")
      const begun = await run(beginPostgres(pool), environment, active)
      if (begun.kind === "failure")
        return fail(begun.error)(environment, active)
      try {
        const used = await run(program(begun.value), environment, active)
        if (used.kind === "failure") {
          const rolledBack = await run(
            rollbackPostgres(begun.value),
            environment,
            active
          )
          return fail(
            rolledBack.kind === "failure" ? rolledBack.error : used.error
          )(environment, active)
        }
        const committed = await run(
          commitPostgres(begun.value),
          environment,
          active
        )
        return committed.kind === "failure"
          ? fail(committed.error)(environment, active)
          : used.value
      } catch (cause) {
        await rollbackAfterInterruption(begun.value, environment)
        throw cause
      }
    } finally {
      await execution?.close()
    }
  }
}

export function openCursor(
  input: PostgresQuery,
  pool: PostgresPool
): Effect<PostgresEnvironment, PostgresError, PostgresCursor> {
  return serviceEffect((environment, context) =>
    environment.postgres.openCursor(pool, input, context)
  )
}

export function fetch<Value>(
  limit: number,
  decoder: PostgresDecoder<Value>,
  cursor: PostgresCursor
): Effect<PostgresEnvironment, PostgresError, ReadonlyArray<Value>> {
  return async (environment, context) => {
    const fetched = await run(
      serviceEffect<
        PostgresEnvironment,
        PostgresError,
        ReadonlyArray<PostgresRow>
      >((services, active) => services.postgres.fetch(cursor, limit, active)),
      environment,
      context
    )
    if (fetched.kind === "failure")
      return fail(fetched.error)(environment, context)
    const result = decodeRows(fetched.value, decoder)
    return result.tag === "DecodeFailure"
      ? fail(rowDecodeFailure(result.value))(environment, context)
      : result.value
  }
}

export function closeCursor(
  cursor: PostgresCursor
): Effect<PostgresEnvironment, PostgresError, Unit> {
  return serviceEffect((environment, context) =>
    environment.postgres.closeCursor(cursor, context)
  )
}

export function closePool(
  pool: PostgresPool
): Effect<PostgresEnvironment, PostgresError, Unit> {
  return serviceEffect((environment, context) =>
    environment.postgres.closePool(pool, context)
  )
}

export function postgresSuccess<Success>(
  value: Success
): ServiceResult<never, Success> {
  return serviceSuccess(value)
}

export function postgresFailure(
  error: PostgresError
): ServiceResult<PostgresError, never> {
  return serviceFailure(error)
}

function beginPostgres(
  pool: PostgresPool
): Effect<PostgresEnvironment, PostgresError, PostgresTransaction> {
  return serviceEffect((environment, context) =>
    environment.postgres.begin(pool, context)
  )
}

function commitPostgres(
  value: PostgresTransaction
): Effect<PostgresEnvironment, PostgresError, Unit> {
  return serviceEffect((environment, context) =>
    environment.postgres.commit(value, context)
  )
}

function rollbackPostgres(
  value: PostgresTransaction
): Effect<PostgresEnvironment, PostgresError, Unit> {
  return serviceEffect((environment, context) =>
    environment.postgres.rollback(value, context)
  )
}

function decodeQueryResult<Value>(
  effect: Effect<PostgresEnvironment, PostgresError, PostgresRawQueryResult>,
  decoder: PostgresDecoder<Value>
): Effect<PostgresEnvironment, PostgresError, PostgresQueryResult<Value>> {
  return async (environment, context) => {
    const result = await run(effect, environment, context)
    if (result.kind === "failure")
      return fail(result.error)(environment, context)
    const rows = decodeRows(result.value.rows, decoder)
    if (rows.tag === "DecodeFailure")
      return fail(rowDecodeFailure(rows.value))(environment, context)
    return Object.freeze({
      rows: rows.value,
      rowCount: result.value.rowCount,
      command: result.value.command,
    })
  }
}

function decodeRows<Value>(
  rows: ReadonlyArray<PostgresRow>,
  decoder: PostgresDecoder<Value>
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
  decode: (value: PostgresValue) => DecodeResult<Value>
): PostgresDecoder<Value> {
  return decoder((row) =>
    Object.hasOwn(row, column)
      ? decode(row[column] as PostgresValue)
      : decodeFailure({ tag: "MissingColumn", value: column })
  )
}

function decoder<Value>(
  run: PostgresDecoderFunction<Value>
): PostgresDecoder<Value> {
  return Object.freeze({ tag: "Decoder" as const, value: run })
}

function runDecoder<Value>(
  decoder: PostgresDecoder<Value>,
  row: PostgresRow
): DecodeResult<Value> {
  if (typeof decoder === "function") return decoder(row)
  if ("run" in decoder) return decoder.run(row)
  return decoder.value(row)
}

const decoded = <Value>(value: Value): DecodeResult<Value> =>
  Object.freeze({ tag: "Decoded", value })

const decodeFailure = (value: PostgresRowDecodeError): DecodeResult<never> =>
  Object.freeze({ tag: "DecodeFailure", value })

const unexpected = (column: string): DecodeResult<never> =>
  decodeFailure({ tag: "UnexpectedColumnType", value: column })

const rowDecodeFailure = (value: PostgresRowDecodeError): PostgresError =>
  Object.freeze({ tag: "RowDecodeFailure", value })

async function rollbackAfterInterruption(
  transaction: PostgresTransaction,
  environment: PostgresEnvironment
): Promise<void> {
  const execution = createEffectExecution()
  try {
    const result = await run(
      rollbackPostgres(transaction),
      environment,
      execution.context
    )
    if (result.kind === "failure") {
      const failure = result.error
      const message =
        failure.tag === "DriverFailure"
          ? failure.value.message
          : failure.value.value
      throw new Error(`PostgreSQL rollback failed: ${message}`)
    }
  } finally {
    await execution.close()
  }
}

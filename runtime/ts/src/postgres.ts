import type { Effect, EffectContext, Unit } from "./effect"
import type { ProviderHandle } from "./provider"
import {
  type ServiceResult,
  serviceEffect,
  serviceFailure,
  serviceSuccess,
} from "./service"

export type PostgresValue = string | number | boolean | null | Uint8Array
export type PostgresRow = Readonly<Record<string, PostgresValue>>
export type PostgresPoolOptions = Readonly<{ connectionString: string }>
export type PostgresQuery = Readonly<{
  text: string
  values: ReadonlyArray<PostgresValue>
}>
export type PostgresOperation =
  | "openPool"
  | "query"
  | "openCursor"
  | "fetch"
  | "closeCursor"
  | "closePool"
export type PostgresError = Readonly<{
  tag: "QueryFailed"
  operation: PostgresOperation
  code: string
  message: string
}>
export type PostgresPool = ProviderHandle
export type PostgresCursor = ProviderHandle
export type Postgres = Readonly<{
  openPool: (
    options: PostgresPoolOptions,
    context: EffectContext
  ) => Promise<ServiceResult<PostgresError, PostgresPool>>
  query: (
    pool: PostgresPool,
    query: PostgresQuery,
    context: EffectContext
  ) => Promise<ServiceResult<PostgresError, ReadonlyArray<PostgresRow>>>
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

export function openPostgresPool(
  options: PostgresPoolOptions
): Effect<PostgresEnvironment, PostgresError, PostgresPool> {
  return serviceEffect((environment, context) =>
    environment.postgres.openPool(options, context)
  )
}

export function queryPostgres(
  pool: PostgresPool,
  query: PostgresQuery
): Effect<PostgresEnvironment, PostgresError, ReadonlyArray<PostgresRow>> {
  return serviceEffect((environment, context) =>
    environment.postgres.query(pool, query, context)
  )
}

export function openPostgresCursor(
  pool: PostgresPool,
  query: PostgresQuery
): Effect<PostgresEnvironment, PostgresError, PostgresCursor> {
  return serviceEffect((environment, context) =>
    environment.postgres.openCursor(pool, query, context)
  )
}

export function fetchPostgresRows(
  cursor: PostgresCursor,
  limit: number
): Effect<PostgresEnvironment, PostgresError, ReadonlyArray<PostgresRow>> {
  return serviceEffect((environment, context) =>
    environment.postgres.fetch(cursor, limit, context)
  )
}

export function closePostgresCursor(
  cursor: PostgresCursor
): Effect<PostgresEnvironment, PostgresError, Unit> {
  return serviceEffect((environment, context) =>
    environment.postgres.closeCursor(cursor, context)
  )
}

export function closePostgresPool(
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

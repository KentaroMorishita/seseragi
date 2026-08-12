import type { Effect } from "@seseragi/runtime/effect"
import {
  closePostgresCursor,
  closePostgresPool,
  fetchPostgresRows,
  openPostgresCursor,
  openPostgresPool,
  type PostgresCursor,
  type PostgresEnvironment,
  type PostgresError,
  type PostgresPool,
  type PostgresRow,
  queryPostgres,
} from "@seseragi/runtime/postgres"

export const openFixturePool = (
  connectionString: string
): Effect<PostgresEnvironment, PostgresError, PostgresPool> =>
  openPostgresPool({ connectionString })

export const queryFixture = (
  pool: PostgresPool,
  text = "select id, name from people"
): Effect<PostgresEnvironment, PostgresError, ReadonlyArray<PostgresRow>> =>
  queryPostgres(pool, { text, values: [] })

export const openFixtureCursor = (
  pool: PostgresPool
): Effect<PostgresEnvironment, PostgresError, PostgresCursor> =>
  openPostgresCursor(pool, {
    text: "select id, name from people order by id",
    values: [],
  })

export const fetchFixtureRows = (
  cursor: PostgresCursor,
  limit: number
): Effect<PostgresEnvironment, PostgresError, ReadonlyArray<PostgresRow>> =>
  fetchPostgresRows(cursor, limit)

export const closeFixtureCursor = closePostgresCursor
export const closeFixturePool = closePostgresPool

import type { Effect } from "@seseragi/runtime/effect"
import {
  closeCursor,
  closePool,
  fetch,
  int,
  map2,
  openCursor,
  openPool,
  type PostgresEnvironment,
  type PostgresError,
  type PostgresPool,
  query,
  string,
  transaction,
  transactionQuery,
} from "@seseragi/runtime/postgres"

export type Person = Readonly<{ id: number; name: string }>

export const personDecoder = map2(
  (id: number, name: string): Person => Object.freeze({ id, name }),
  int("id"),
  string("name")
)

export const openFixturePool = (
  connectionString: string
): Effect<PostgresEnvironment, PostgresError, PostgresPool> =>
  openPool({ connectionString, maxConnections: 4 })

export const queryFixture = (
  pool: PostgresPool,
  text = "select id, name from people"
) => query(pool, { text, values: [] }, personDecoder)

export const transactionFixture = (pool: PostgresPool, text: string) =>
  transaction(pool, transactionQuery({ text, values: [] }, personDecoder))

export const openFixtureCursor = (pool: PostgresPool) =>
  openCursor(
    { text: "select id, name from people order by id", values: [] },
    pool
  )

export const fetchFixtureRows = (
  cursor: Parameters<typeof fetch>[2],
  limit: number
) => fetch(limit, personDecoder, cursor)

export const closeFixtureCursor = closeCursor
export const closeFixturePool = closePool

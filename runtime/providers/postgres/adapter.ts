import {
  type ProviderResult,
  providerRuntimeAbi,
} from "@seseragi/runtime/provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
} from "@seseragi/runtime/provider-package"

export type DriverQuery = Readonly<{
  text: string
  values: ReadonlyArray<unknown>
}>
export type DriverCursor = Readonly<{
  read: (limit: number) => Promise<ReadonlyArray<unknown>>
  close: () => Promise<void>
}>
export type DriverClient = Readonly<{
  release: () => void
}>
export type DriverPool = Readonly<{
  query: (
    text: string,
    values: ReadonlyArray<unknown>
  ) => Promise<Readonly<{ rows: ReadonlyArray<unknown> }>>
  connect: () => Promise<DriverClient>
  end: () => Promise<void>
}>
export type PostgresDriver = Readonly<{
  createPool: (connectionString: string) => DriverPool
  openCursor: (client: DriverClient, query: DriverQuery) => DriverCursor
}>

type PoolToken = {
  readonly pool: DriverPool
  readonly cursors: Set<CursorToken>
  closeCompletion?: Promise<void>
}
type CursorToken = {
  readonly parent: PoolToken
  readonly client: DriverClient
  readonly cursor: DriverCursor
  closeCompletion?: Promise<void>
}
type Operation =
  | "openPool"
  | "query"
  | "openCursor"
  | "fetch"
  | "closeCursor"
  | "closePool"

export function createPostgresProvider(
  driver: PostgresDriver
): ProviderPackageEntry {
  const pools = new Set<PoolToken>()
  const cursors = new Set<CursorToken>()
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: "seseragi/runtime-postgres#pg",
    service: "acme/postgres::Postgres",
    targets: ["bun-process", "node-process"],
    operations: {
      async openPool(value) {
        try {
          const options = dataRecord(value, ["connectionString"])
          if (typeof options.connectionString !== "string") {
            throw new TypeError("PostgreSQL connection string must be a string")
          }
          const token: PoolToken = {
            pool: driver.createPool(options.connectionString),
            cursors: new Set(),
          }
          pools.add(token)
          return { kind: "success", value: token }
        } catch (cause) {
          return failure("openPool", cause)
        }
      },
      async query(value) {
        try {
          const input = dataRecord(value, ["pool", "query"])
          const pool = ownedPool(input.pool, pools)
          ensureOpen(pool.closeCompletion, "PostgreSQL pool")
          const query = queryInput(input.query)
          const result = await pool.pool.query(query.text, query.values)
          return { kind: "success", value: Object.freeze([...result.rows]) }
        } catch (cause) {
          return failure("query", cause)
        }
      },
      async openCursor(value) {
        let client: DriverClient | undefined
        try {
          const input = dataRecord(value, ["pool", "query"])
          const pool = ownedPool(input.pool, pools)
          ensureOpen(pool.closeCompletion, "PostgreSQL pool")
          const query = queryInput(input.query)
          client = await pool.pool.connect()
          const token: CursorToken = {
            parent: pool,
            client,
            cursor: driver.openCursor(client, query),
          }
          pool.cursors.add(token)
          cursors.add(token)
          return { kind: "success", value: token }
        } catch (cause) {
          client?.release()
          return failure("openCursor", cause)
        }
      },
      async fetch(value) {
        try {
          const input = dataRecord(value, ["cursor", "limit"])
          const cursor = ownedCursor(input.cursor, cursors)
          ensureOpen(cursor.closeCompletion, "PostgreSQL cursor")
          if (
            !Number.isSafeInteger(input.limit) ||
            (input.limit as number) <= 0 ||
            (input.limit as number) > 0x7fff_ffff
          ) {
            throw new RangeError("PostgreSQL cursor fetch limit is invalid")
          }
          const rows = await cursor.cursor.read(input.limit as number)
          return { kind: "success", value: Object.freeze([...rows]) }
        } catch (cause) {
          return failure("fetch", cause)
        }
      },
      async closeCursor(value) {
        try {
          await closeCursor(ownedCursor(value, cursors))
          return { kind: "success", value: undefined }
        } catch (cause) {
          return failure("closeCursor", cause)
        }
      },
      async closePool(value) {
        try {
          await closePool(ownedPool(value, pools))
          return { kind: "success", value: undefined }
        } catch (cause) {
          return failure("closePool", cause)
        }
      },
    },
    shutdown: async () => {
      for (const pool of [...pools].reverse()) await closePool(pool)
      pools.clear()
      cursors.clear()
    },
  })

  function closeCursor(token: CursorToken): Promise<void> {
    token.closeCompletion ??= (async () => {
      try {
        await token.cursor.close()
      } finally {
        token.client.release()
        token.parent.cursors.delete(token)
      }
    })()
    return token.closeCompletion
  }

  function closePool(token: PoolToken): Promise<void> {
    token.closeCompletion ??= (async () => {
      let firstFailure: unknown
      for (const cursor of [...token.cursors].reverse()) {
        try {
          await closeCursor(cursor)
        } catch (cause) {
          firstFailure ??= cause
        }
      }
      try {
        await token.pool.end()
      } catch (cause) {
        firstFailure ??= cause
      }
      if (firstFailure !== undefined) throw firstFailure
    })()
    return token.closeCompletion
  }
}

function ownedPool(value: unknown, pools: Set<PoolToken>): PoolToken {
  if (
    typeof value !== "object" ||
    value === null ||
    !pools.has(value as PoolToken)
  ) {
    throw new TypeError("PostgreSQL pool is not owned by this provider")
  }
  return value as PoolToken
}

function ownedCursor(value: unknown, cursors: Set<CursorToken>): CursorToken {
  if (
    typeof value !== "object" ||
    value === null ||
    !cursors.has(value as CursorToken)
  ) {
    throw new TypeError("PostgreSQL cursor is not owned by this provider")
  }
  return value as CursorToken
}

function ensureOpen(completion: Promise<void> | undefined, name: string): void {
  if (completion !== undefined) throw resourceClosed(`${name} is closed`)
}

function queryInput(value: unknown): DriverQuery {
  const query = dataRecord(value, ["text", "values"])
  if (typeof query.text !== "string" || !Array.isArray(query.values)) {
    throw new TypeError("PostgreSQL query input is invalid")
  }
  return Object.freeze({
    text: query.text,
    values: Object.freeze([...query.values]),
  })
}

function failure(operation: Operation, cause: unknown): ProviderResult {
  const error = cause as { code?: unknown; message?: unknown }
  return {
    kind: "failure",
    failure: Object.freeze({
      tag: "QueryFailed",
      operation,
      code: typeof error?.code === "string" ? error.code : "POSTGRES_ERROR",
      message:
        typeof error?.message === "string"
          ? error.message
          : "PostgreSQL failed",
    }),
  }
}

function resourceClosed(message: string): Error & { code: string } {
  return Object.assign(new Error(message), { code: "RESOURCE_CLOSED" })
}

function dataRecord(
  value: unknown,
  keys: ReadonlyArray<string>
): Record<string, unknown> {
  if (
    typeof value !== "object" ||
    value === null ||
    ![Object.prototype, null].includes(Object.getPrototypeOf(value))
  ) {
    throw new TypeError("PostgreSQL provider input must be a plain record")
  }
  const actual = Reflect.ownKeys(value)
  if (
    actual.length !== keys.length ||
    actual.some((key) => typeof key !== "string" || !keys.includes(key))
  ) {
    throw new TypeError("PostgreSQL provider input shape is invalid")
  }
  const record: Record<string, unknown> = {}
  for (const key of keys) {
    const descriptor = Object.getOwnPropertyDescriptor(value, key)
    if (descriptor === undefined || !("value" in descriptor)) {
      throw new TypeError("PostgreSQL provider input must use data fields")
    }
    record[key] = descriptor.value
  }
  return record
}

import {
  type ProviderResult,
  providerRuntimeAbi,
} from "@seseragi/runtime/provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
} from "@seseragi/runtime/provider-package"

export type DriverValue = string | number | bigint | null | Uint8Array
export type DriverStatement = Readonly<{
  sql: string
  values: ReadonlyArray<DriverValue>
}>
export type DriverExecuteResult = Readonly<{
  changes: number | bigint
  lastInsertRowId: number | bigint
}>
export type DriverDatabase = Readonly<{
  query: (statement: DriverStatement) => ReadonlyArray<unknown>
  execute: (statement: DriverStatement) => DriverExecuteResult
  beginImmediate: () => void
  commit: () => void
  rollback: () => void
  close: () => void
}>
export type SqliteDriver = Readonly<{
  openMemory: (busyTimeoutMillis: number) => DriverDatabase
  openFile: (config: {
    readonly filename: string
    readonly readOnly: boolean
    readonly create: boolean
    readonly busyTimeoutMillis: number
  }) => DriverDatabase
}>

type DatabaseToken = {
  readonly database: DriverDatabase
  transaction: TransactionToken | undefined
  closeCompletion?: Promise<void>
}
type TransactionToken = {
  readonly parent: DatabaseToken
  closeCompletion?: Promise<void>
}
type Operation =
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

export function createSqliteProvider(
  driver: SqliteDriver
): ProviderPackageEntry {
  const databases = new Set<DatabaseToken>()
  const transactions = new Set<TransactionToken>()
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: "seseragi/runtime-sqlite#bun",
    service: "seseragi/sqlite::Sqlite",
    targets: ["bun-process"],
    operations: {
      async openMemory(value) {
        try {
          const busyTimeoutMillis = timeout(value)
          const token: DatabaseToken = {
            database: driver.openMemory(busyTimeoutMillis),
            transaction: undefined,
          }
          databases.add(token)
          return { kind: "success", value: token }
        } catch (cause) {
          return failure("openMemory", cause)
        }
      },
      async openFile(value) {
        try {
          const config = fileConfig(value)
          const token: DatabaseToken = {
            database: driver.openFile(config),
            transaction: undefined,
          }
          databases.add(token)
          return { kind: "success", value: token }
        } catch (cause) {
          return failure("openFile", cause)
        }
      },
      async query(value) {
        let rows: ReadonlyArray<unknown>
        try {
          const input = dataRecord(value, ["database", "statement"])
          const database = ownedDatabase(input.database, databases)
          ensureAvailable(database)
          rows = database.database.query(statementInput(input.statement))
        } catch (cause) {
          return failure("query", cause)
        }
        return { kind: "success", value: queryResult(rows) }
      },
      async execute(value) {
        let result: DriverExecuteResult
        try {
          const input = dataRecord(value, ["database", "statement"])
          const database = ownedDatabase(input.database, databases)
          ensureAvailable(database)
          result = database.database.execute(statementInput(input.statement))
        } catch (cause) {
          return failure("execute", cause)
        }
        return { kind: "success", value: executeResult(result) }
      },
      async begin(value) {
        try {
          const database = ownedDatabase(value, databases)
          ensureAvailable(database)
          database.database.beginImmediate()
          const token: TransactionToken = { parent: database }
          database.transaction = token
          transactions.add(token)
          return { kind: "success", value: token }
        } catch (cause) {
          return failure("begin", cause)
        }
      },
      async transactionQuery(value) {
        let rows: ReadonlyArray<unknown>
        try {
          const input = dataRecord(value, ["transaction", "statement"])
          const transaction = ownedTransaction(
            input.transaction,
            databases,
            transactions
          )
          ensureTransactionOpen(transaction)
          rows = transaction.parent.database.query(
            statementInput(input.statement)
          )
        } catch (cause) {
          return failure("transactionQuery", cause)
        }
        return { kind: "success", value: queryResult(rows) }
      },
      async transactionExecute(value) {
        let result: DriverExecuteResult
        try {
          const input = dataRecord(value, ["transaction", "statement"])
          const transaction = ownedTransaction(
            input.transaction,
            databases,
            transactions
          )
          ensureTransactionOpen(transaction)
          result = transaction.parent.database.execute(
            statementInput(input.statement)
          )
        } catch (cause) {
          return failure("transactionExecute", cause)
        }
        return { kind: "success", value: executeResult(result) }
      },
      async commit(value) {
        try {
          await closeTransaction(
            ownedTransaction(value, databases, transactions),
            "commit"
          )
          return { kind: "success", value: undefined }
        } catch (cause) {
          return failure("commit", cause)
        }
      },
      async rollback(value) {
        try {
          await closeTransaction(
            ownedTransaction(value, databases, transactions),
            "rollback"
          )
          return { kind: "success", value: undefined }
        } catch (cause) {
          return failure("rollback", cause)
        }
      },
      async close(value) {
        try {
          await closeDatabase(ownedDatabase(value, databases))
          return { kind: "success", value: undefined }
        } catch (cause) {
          return failure("close", cause)
        }
      },
    },
    shutdown: async () => {
      for (const database of [...databases].reverse()) {
        await closeDatabase(database)
      }
      transactions.clear()
      databases.clear()
    },
  })

  function closeTransaction(
    token: TransactionToken,
    operation: "commit" | "rollback"
  ): Promise<void> {
    token.closeCompletion ??= Promise.resolve().then(() => {
      try {
        token.parent.database[operation]()
      } catch (cause) {
        if (operation === "commit") {
          try {
            token.parent.database.rollback()
          } catch {
            // Preserve the commit failure. Database cleanup remains the final
            // fallback when the host cannot roll the transaction back either.
          }
        }
        throw cause
      } finally {
        token.parent.transaction = undefined
        transactions.delete(token)
      }
    })
    return token.closeCompletion
  }

  function closeDatabase(token: DatabaseToken): Promise<void> {
    token.closeCompletion ??= (async () => {
      let firstFailure: unknown
      if (token.transaction !== undefined) {
        try {
          await closeTransaction(token.transaction, "rollback")
        } catch (cause) {
          firstFailure = cause
        }
      }
      try {
        token.database.close()
      } catch (cause) {
        firstFailure ??= cause
      } finally {
        databases.delete(token)
      }
      if (firstFailure !== undefined) throw firstFailure
    })()
    return token.closeCompletion
  }
}

function ownedDatabase(
  value: unknown,
  databases: Set<DatabaseToken>
): DatabaseToken {
  if (
    typeof value !== "object" ||
    value === null ||
    !databases.has(value as DatabaseToken)
  ) {
    throw resourceError(
      "RESOURCE_NOT_OWNED",
      "SQLite database is not owned by this provider"
    )
  }
  return value as DatabaseToken
}

function ownedTransaction(
  value: unknown,
  databases: Set<DatabaseToken>,
  transactions: Set<TransactionToken>
): TransactionToken {
  if (typeof value !== "object" || value === null) {
    throw resourceError("RESOURCE_NOT_OWNED", "SQLite transaction is invalid")
  }
  const token = value as TransactionToken
  if (
    !databases.has(token.parent) ||
    !transactions.has(token) ||
    token.parent.transaction !== token
  ) {
    throw resourceError(
      "RESOURCE_NOT_OWNED",
      "SQLite transaction is not owned by this provider"
    )
  }
  return token
}

function ensureAvailable(database: DatabaseToken): void {
  if (database.closeCompletion !== undefined) {
    throw resourceError("RESOURCE_CLOSED", "SQLite database is closed")
  }
  if (database.transaction !== undefined) {
    throw resourceError(
      "TRANSACTION_ACTIVE",
      "SQLite database has an active transaction"
    )
  }
}

function ensureTransactionOpen(transaction: TransactionToken): void {
  if (transaction.closeCompletion !== undefined) {
    throw resourceError("RESOURCE_CLOSED", "SQLite transaction is closed")
  }
  if (transaction.parent.closeCompletion !== undefined) {
    throw resourceError("RESOURCE_CLOSED", "SQLite database is closed")
  }
}

function fileConfig(value: unknown) {
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
    (config.readOnly && config.create)
  ) {
    throw new TypeError("SQLite file configuration is invalid")
  }
  return Object.freeze({
    filename: config.filename,
    readOnly: config.readOnly,
    create: config.create,
    busyTimeoutMillis: timeout(config.busyTimeoutMillis),
  })
}

function timeout(value: unknown): number {
  if (
    !Number.isSafeInteger(value) ||
    (value as number) < 0 ||
    (value as number) > 0x7fff_ffff
  ) {
    throw new RangeError("SQLite busy timeout is invalid")
  }
  return value as number
}

function statementInput(value: unknown): DriverStatement {
  const statement = dataRecord(value, ["sql", "values"])
  if (
    typeof statement.sql !== "string" ||
    statement.sql.length === 0 ||
    !Array.isArray(statement.values)
  ) {
    throw new TypeError("SQLite statement input is invalid")
  }
  return Object.freeze({
    sql: statement.sql,
    values: Object.freeze(statement.values.map(driverValue)),
  })
}

function driverValue(value: unknown): DriverValue {
  if (
    value === null ||
    typeof value === "string" ||
    (typeof value === "number" && Number.isFinite(value))
  ) {
    return value
  }
  if (value instanceof Uint8Array) return new Uint8Array(value)
  throw new TypeError("SQLite bind value is invalid")
}

function queryResult(rows: ReadonlyArray<unknown>) {
  return Object.freeze({ rows: Object.freeze(rows.map(snapshotRow)) })
}

function snapshotRow(value: unknown): unknown {
  if (!isPlainRecord(value)) return value
  const row: Record<string, unknown> = {}
  for (const key of Reflect.ownKeys(value)) {
    if (typeof key !== "string") return value
    const descriptor = Object.getOwnPropertyDescriptor(value, key)
    if (
      descriptor === undefined ||
      !("value" in descriptor) ||
      !descriptor.enumerable
    ) {
      return value
    }
    row[key] = boundaryValue(descriptor.value)
  }
  return Object.freeze(row)
}

function boundaryValue(value: unknown): unknown {
  if (typeof value === "bigint") return safeInt(value)
  if (
    value === null ||
    typeof value === "string" ||
    (typeof value === "number" && Number.isFinite(value))
  ) {
    return value
  }
  if (value instanceof Uint8Array) return new Uint8Array(value)
  return value
}

function executeResult(value: DriverExecuteResult) {
  return Object.freeze({
    changes: safeInt(value.changes),
    lastInsertRowId: safeInt(value.lastInsertRowId),
  })
}

function safeInt(value: number | bigint): number | bigint {
  if (typeof value === "number") {
    if (Number.isSafeInteger(value)) return value
  } else if (
    value >= BigInt(Number.MIN_SAFE_INTEGER) &&
    value <= BigInt(Number.MAX_SAFE_INTEGER)
  ) {
    return Number(value)
  }
  return value
}

function failure(operation: Operation, cause: unknown): ProviderResult {
  const error = cause as { code?: unknown; message?: unknown }
  return {
    kind: "failure",
    failure: Object.freeze({
      tag: "QueryFailed",
      operation,
      code: typeof error?.code === "string" ? error.code : "SQLITE_ERROR",
      message:
        typeof error?.message === "string" ? error.message : "SQLite failed",
    }),
  }
}

function resourceError(
  code: string,
  message: string
): Error & { code: string } {
  return Object.assign(new Error(message), { code })
}

function dataRecord(
  value: unknown,
  keys: ReadonlyArray<string>
): Record<string, unknown> {
  if (!isPlainRecord(value)) {
    throw new TypeError("SQLite provider input must be a plain record")
  }
  const actual = Reflect.ownKeys(value)
  if (
    actual.length !== keys.length ||
    actual.some((key) => typeof key !== "string" || !keys.includes(key))
  ) {
    throw new TypeError("SQLite provider input shape is invalid")
  }
  const record: Record<string, unknown> = {}
  for (const key of keys) {
    const descriptor = Object.getOwnPropertyDescriptor(value, key)
    if (descriptor === undefined || !("value" in descriptor)) {
      throw new TypeError("SQLite provider input must use data fields")
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

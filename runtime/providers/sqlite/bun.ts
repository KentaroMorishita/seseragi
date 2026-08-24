import { Database } from "bun:sqlite"
import {
  createSqliteProvider,
  type DriverDatabase,
  type DriverStatement,
} from "./adapter"

export const provider = createSqliteProvider({
  openMemory: (busyTimeoutMillis) =>
    openDatabase(":memory:", { busyTimeoutMillis }),
  openFile: (config) => openDatabase(config.filename, config),
})

function openDatabase(
  filename: string,
  options: {
    readonly readOnly?: boolean
    readonly create?: boolean
    readonly busyTimeoutMillis: number
  }
): DriverDatabase {
  const database = new Database(filename, {
    readonly: options.readOnly ?? false,
    readwrite: !(options.readOnly ?? false),
    create: options.create ?? true,
    safeIntegers: true,
    strict: true,
  })
  try {
    database.exec(`PRAGMA busy_timeout = ${options.busyTimeoutMillis}`)
  } catch (cause) {
    database.close()
    throw cause
  }
  return Object.freeze({
    query: (statement) => query(database, statement),
    execute: (statement) => {
      const result = execute(database, statement)
      return Object.freeze({
        changes: result.changes,
        lastInsertRowId: result.lastInsertRowid,
      })
    },
    beginImmediate: () => {
      database.exec("BEGIN IMMEDIATE")
    },
    commit: () => {
      database.exec("COMMIT")
    },
    rollback: () => {
      database.exec("ROLLBACK")
    },
    close: () => database.close(true),
  })
}

function query(
  database: Database,
  statement: DriverStatement
): ReadonlyArray<unknown> {
  const prepared = database.prepare(statement.sql)
  try {
    return prepared.all(...statement.values)
  } finally {
    prepared.finalize()
  }
}

function execute(database: Database, statement: DriverStatement) {
  const prepared = database.prepare(statement.sql)
  try {
    return prepared.run(...statement.values)
  } finally {
    prepared.finalize()
  }
}

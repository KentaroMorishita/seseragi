import pg from "pg"
import Cursor from "pg-cursor"
import {
  createPostgresProvider,
  type DriverClient,
  type DriverQuery,
} from "./adapter"

export const provider = createPostgresProvider({
  createPool: (config) =>
    new pg.Pool({
      connectionString: config.connectionString,
      max: config.maxConnections,
    }),
  openCursor(client: DriverClient, query: DriverQuery) {
    return (client as pg.PoolClient).query(
      new Cursor(query.text, query.values as unknown[])
    )
  },
})

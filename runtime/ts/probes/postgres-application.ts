import type { Effect } from "@seseragi/runtime/effect"
import {
  bool,
  closeCursor,
  closePool,
  fetch,
  int,
  openCursor,
  openPool,
  type PostgresDecoder,
  type PostgresEnvironment,
  type PostgresError,
  type PostgresPool,
  type PostgresRow,
  query,
  string,
  transaction,
  transactionQuery,
} from "@seseragi/runtime/postgres"

export type Person = Readonly<{
  id: number
  name: string
  active: boolean
}>

type DecoderFunction<Value> = Extract<
  PostgresDecoder<Value>,
  (row: PostgresRow) => unknown
>
type DecodeResult<Value> = ReturnType<DecoderFunction<Value>>

const runDecoder = <Value>(
  value: PostgresDecoder<Value>,
  row: PostgresRow
): DecodeResult<Value> => {
  if (typeof value === "function") return value(row)
  if ("run" in value) return value.run(row)
  return value.value(row)
}

const decoder = <Value>(
  run: DecoderFunction<Value>
): PostgresDecoder<Value> =>
  Object.freeze({ tag: "Decoder" as const, value: run })

const decoded = <Value>(value: Value): DecodeResult<Value> =>
  Object.freeze({ tag: "Decoded" as const, value }) as DecodeResult<Value>

const mapDecoder = <Input, Output>(
  transform: (value: Input) => Output,
  input: PostgresDecoder<Input>
): PostgresDecoder<Output> =>
  decoder((row) => {
    const result = runDecoder(input, row)
    return result.tag === "DecodeFailure"
      ? result
      : decoded(transform(result.value))
  })

const applyDecoder = <Input, Output>(
  wrapped: PostgresDecoder<(value: Input) => Output>,
  input: PostgresDecoder<Input>
): PostgresDecoder<Output> =>
  decoder((row) => {
    const transform = runDecoder(wrapped, row)
    if (transform.tag === "DecodeFailure") return transform
    const result = runDecoder(input, row)
    return result.tag === "DecodeFailure"
      ? result
      : decoded(transform.value(result.value))
  })

const person =
  (id: number) =>
  (name: string) =>
  (active: boolean): Person =>
    Object.freeze({ id, name, active })

export const personDecoder = applyDecoder(
  applyDecoder(mapDecoder(person, int("id")), string("name")),
  bool("active")
)

export const openFixturePool = (
  connectionString: string
): Effect<PostgresEnvironment, PostgresError, PostgresPool> =>
  openPool({ connectionString, maxConnections: 4 })

export const queryFixture = (
  pool: PostgresPool,
  text = "select id, name, active from people"
) => query(pool, { text, values: [] }, personDecoder)

export const transactionFixture = (pool: PostgresPool, text: string) =>
  transaction(pool, transactionQuery({ text, values: [] }, personDecoder))

export const openFixtureCursor = (pool: PostgresPool) =>
  openCursor(
    { text: "select id, name, active from people order by id", values: [] },
    pool
  )

export const fetchFixtureRows = (
  cursor: Parameters<typeof fetch>[2],
  limit: number
) => fetch(limit, personDecoder, cursor)

export const closeFixtureCursor = closeCursor
export const closeFixturePool = closePool

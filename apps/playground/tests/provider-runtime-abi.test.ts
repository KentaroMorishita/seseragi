import { describe, expect, test } from "bun:test"
import { createEffectExecution } from "../../../runtime/ts/src/effect"
import {
  assertProviderRuntimeAbi,
  decodeProviderValue,
  encodeProviderValue,
  invokeProviderOperation,
  openProviderSubscription,
  ProviderCodecRegistry,
  type ProviderLogicalType,
  type ProviderOperationContract,
  providerRuntimeAbi,
} from "../../../runtime/ts/src/provider"

const codecs = new ProviderCodecRegistry()

const unit = { kind: "unit" } as const
const never = { kind: "never" } as const
const int = { kind: "primitive", name: "int" } as const
const float = { kind: "primitive", name: "float" } as const
const bytes = { kind: "primitive", name: "bytes" } as const
const string = { kind: "primitive", name: "string" } as const

function operation(
  identity: string,
  input: ProviderLogicalType,
  success: ProviderLogicalType,
  failure: ProviderLogicalType = never,
  kind: ProviderOperationContract["kind"] = "one-shot"
): ProviderOperationContract {
  return { identity, kind, input, success, failure }
}

describe("TypeScript Provider Runtime ABI v1", () => {
  test("linearizes subscription demand and unsubscribes exactly once", async () => {
    let registrations = 0
    let demands = 0
    let unsubscribes = 0
    const source = openProviderSubscription({
      provider: "fixture/runtime-bun#stream",
      service: "fixture/stream::Values",
      operation: operation(
        "fixture/stream::Values#subscribe",
        unit,
        int,
        string,
        "subscription"
      ),
      entry: {
        subscribe(_input: unknown, observerValue: unknown) {
          registrations += 1
          const observer = observerValue as {
            next(value: unknown): void
          }
          return {
            demand(count: number) {
              demands += count
              observer.next(42)
            },
            unsubscribe() {
              unsubscribes += 1
            },
          }
        },
      },
      input: undefined,
      codecs,
    })
    const execution = createEffectExecution()

    expect(registrations).toBe(1)
    expect(demands).toBe(0)
    expect(await source.pull(execution.context)).toEqual({
      done: false,
      value: 42,
    })
    expect(demands).toBe(1)
    await source.close()
    await source.close()
    expect(unsubscribes).toBe(1)
  })

  test("classifies over-emission as a result defect and discards late events", async () => {
    let observer:
      | {
          next(value: unknown): void
          complete(): void
        }
      | undefined
    const source = openProviderSubscription({
      provider: "fixture/runtime-node#stream",
      service: "fixture/stream::Values",
      operation: operation(
        "fixture/stream::Values#subscribe",
        unit,
        int,
        string,
        "subscription"
      ),
      entry: {
        subscribe(_input: unknown, observerValue: unknown) {
          observer = observerValue as typeof observer
          return {
            demand() {
              observer?.next(1)
              observer?.next(2)
            },
            unsubscribe() {},
          }
        },
      },
      input: undefined,
      codecs,
    })
    const execution = createEffectExecution()

    expect(await source.pull(execution.context)).toEqual({
      done: false,
      value: 1,
    })
    const defect = await source.pull(execution.context).catch((error) => error)
    expect(defect).toBeInstanceOf(Error)
    expect(defect).toMatchObject({ stage: "result" })
    observer?.next(3)
    observer?.complete()
    await source.close()
  })

  test("requires the exact independent ABI handshake", () => {
    expect(assertProviderRuntimeAbi({ ...providerRuntimeAbi })).toBe(
      providerRuntimeAbi
    )
    for (const invalid of [
      { ...providerRuntimeAbi, abiMajor: 2 },
      { ...providerRuntimeAbi, backend: "javascript" },
      { ...providerRuntimeAbi, identity: "@seseragi/runtime" },
      { ...providerRuntimeAbi, extra: true },
    ]) {
      expect(() => assertProviderRuntimeAbi(invalid)).toThrow()
    }
  })

  test("validates primitives and keeps numeric edge values distinct", () => {
    expect(encodeProviderValue(unit, undefined, codecs)).toBeUndefined()
    expect(() => encodeProviderValue(unit, null, codecs)).toThrow()
    expect(() => decodeProviderValue(string, undefined, codecs)).toThrow()
    expect(() => decodeProviderValue(string, null, codecs)).toThrow()
    expect(() => decodeProviderValue(never, "impossible", codecs)).toThrow()
    expect(Object.is(decodeProviderValue(float, -0, codecs), -0)).toBe(true)
    expect(Number.isNaN(decodeProviderValue(float, Number.NaN, codecs))).toBe(
      true
    )
    expect(decodeProviderValue(float, Number.POSITIVE_INFINITY, codecs)).toBe(
      Number.POSITIVE_INFINITY
    )
    expect(Object.is(decodeProviderValue(int, -0, codecs), 0)).toBe(true)
    expect(() => decodeProviderValue(int, 1.5, codecs)).toThrow()
    expect(() => decodeProviderValue(int, Number.MAX_VALUE, codecs)).toThrow()
  })

  test("copies Bytes, arrays, and closed records without invoking getters", () => {
    const sourceBytes = new Uint8Array([1, 2, 3])
    const copiedBytes = decodeProviderValue(
      bytes,
      sourceBytes,
      codecs
    ) as Uint8Array
    const encodedBytes = encodeProviderValue(
      bytes,
      sourceBytes,
      codecs
    ) as Uint8Array
    sourceBytes[0] = 9
    expect([...copiedBytes]).toEqual([1, 2, 3])
    expect([...encodedBytes]).toEqual([1, 2, 3])

    const arrayType = { kind: "array", items: int } as const
    const sourceArray = [1, 2]
    const copiedArray = decodeProviderValue(
      arrayType,
      sourceArray,
      codecs
    ) as ReadonlyArray<number>
    sourceArray[0] = 9
    expect(copiedArray).toEqual([1, 2])
    expect(Object.isFrozen(copiedArray)).toBe(true)
    const sparse = [1, 2, 3]
    delete sparse[1]
    expect(() => decodeProviderValue(arrayType, sparse, codecs)).toThrow()

    const recordType = {
      kind: "record",
      fields: [
        { name: "name", type: string },
        { name: "scores", type: arrayType },
      ],
    } as const
    const sourceRecord = { name: "Ada", scores: [1, 2] }
    const copiedRecord = decodeProviderValue(
      recordType,
      sourceRecord,
      codecs
    ) as Readonly<{ name: string; scores: ReadonlyArray<number> }>
    sourceRecord.scores[0] = 9
    expect(copiedRecord).toEqual({ name: "Ada", scores: [1, 2] })
    expect(Object.isFrozen(copiedRecord)).toBe(true)

    let getterRead = false
    const accessor = Object.defineProperty({}, "name", {
      enumerable: true,
      get() {
        getterRead = true
        return "Ada"
      },
    })
    expect(() =>
      decodeProviderValue(
        { kind: "record", fields: [{ name: "name", type: string }] },
        accessor,
        codecs
      )
    ).toThrow()
    expect(getterRead).toBe(false)
    expect(() =>
      decodeProviderValue(recordType, { ...sourceRecord, extra: true }, codecs)
    ).toThrow()
    expect(() =>
      decodeProviderValue(recordType, { name: "Ada" }, codecs)
    ).toThrow()
    const symbolRecord = { name: "Ada", scores: [1, 2] }
    Object.defineProperty(symbolRecord, Symbol("hidden"), { value: true })
    expect(() =>
      decodeProviderValue(recordType, symbolRecord, codecs)
    ).toThrow()

    const unitField = {
      kind: "record",
      fields: [{ name: "value", type: unit }],
    } as const
    expect(
      decodeProviderValue(unitField, { value: undefined }, codecs)
    ).toEqual({ value: undefined })
    expect(() => decodeProviderValue(unitField, {}, codecs)).toThrow()
  })

  test("routes named values only through registered bridge codecs", () => {
    const instant = { kind: "named", identity: "std/time::Instant" } as const
    expect(() => decodeProviderValue(instant, 42, codecs)).toThrow()

    const namedCodecs = new ProviderCodecRegistry([
      {
        identity: "std/time::Instant",
        encode(value) {
          if (
            typeof value !== "object" ||
            value === null ||
            !("ticks" in value)
          ) {
            throw new TypeError("Instant must contain ticks")
          }
          return (value as { ticks: number }).ticks
        },
        decode(value) {
          if (value === null) return Object.freeze({ kind: "origin" })
          if (typeof value !== "number") throw new TypeError("invalid ticks")
          return Object.freeze({ kind: "instant", ticks: value })
        },
      },
    ])
    expect(decodeProviderValue(instant, 42, namedCodecs)).toEqual({
      kind: "instant",
      ticks: 42,
    })
    expect(decodeProviderValue(instant, null, namedCodecs)).toEqual({
      kind: "origin",
    })
    expect(() => decodeProviderValue(instant, undefined, namedCodecs)).toThrow()
    expect(encodeProviderValue(instant, { ticks: 42 }, namedCodecs)).toBe(42)
  })

  test("calls one Promise operation with one encoded argument", async () => {
    let calls = 0
    let argumentCount = 0
    let argument: unknown = "not-called"
    const entry = {
      async now(...values: unknown[]) {
        calls += 1
        argumentCount = values.length
        ;[argument] = values
        return { kind: "success", value: 42 }
      },
    }
    const outcome = await invokeProviderOperation({
      provider: "seseragi/runtime-bun#clock",
      service: "std/clock::Clock",
      operation: operation("std/clock::Clock#now", unit, int),
      entry,
      input: undefined,
      codecs,
    })
    expect(outcome).toEqual({ kind: "success", value: 42 })
    expect(calls).toBe(1)
    expect(argumentCount).toBe(1)
    expect(argument).toBeUndefined()
  })

  test("keeps typed failure separate from bridge defects", async () => {
    const failureType = {
      kind: "named",
      identity: "std/fs::FileError",
    } as const
    const failureCodecs = new ProviderCodecRegistry([
      {
        identity: failureType.identity,
        encode: (value) => value,
        decode(value) {
          if (typeof value !== "string") throw new TypeError("invalid error")
          return Object.freeze({ tag: "FileError", code: value })
        },
      },
    ])
    const contract = operation(
      "std/fs::FileSystem#close",
      unit,
      unit,
      failureType
    )
    const failure = await invokeProviderOperation({
      provider: "seseragi/runtime-node#filesystem",
      service: "std/fs::FileSystem",
      operation: contract,
      entry: {
        async close() {
          return { kind: "failure", failure: "permission" }
        },
      },
      input: undefined,
      codecs: failureCodecs,
    })
    expect(failure).toEqual({
      kind: "failure",
      failure: { tag: "FileError", code: "permission" },
    })

    const cause = { host: "rejected" }
    const rejected = await invokeProviderOperation({
      provider: "seseragi/runtime-node#filesystem",
      service: "std/fs::FileSystem",
      operation: contract,
      entry: { close: () => Promise.reject(cause) },
      input: undefined,
      codecs: failureCodecs,
    })
    expect(rejected.kind).toBe("defect")
    if (rejected.kind === "defect") {
      expect(rejected.defect.stage).toBe("call")
      expect(rejected.defect.cause).toBe(cause)
      expect(rejected.defect.provider).toBe("seseragi/runtime-node#filesystem")
    }

    const hostCause = new Error("host stack fixture")
    const stacked = await invokeProviderOperation({
      provider: "seseragi/runtime-node#filesystem",
      service: "std/fs::FileSystem",
      operation: contract,
      entry: { close: () => Promise.reject(hostCause) },
      input: undefined,
      codecs: failureCodecs,
      source: { path: "src/main.ssrg", start: 12, end: 24 },
    })
    expect(stacked.kind).toBe("defect")
    if (stacked.kind === "defect") {
      expect(stacked.defect.frames[0]).toEqual({
        kind: "seseragi",
        path: "src/main.ssrg",
        start: 12,
        end: 24,
      })
      expect(stacked.defect.frames[1]).toEqual(
        expect.objectContaining({
          kind: "host",
          stack: expect.stringContaining("host stack fixture"),
        })
      )
    }

    const synchronous = await invokeProviderOperation({
      provider: "seseragi/runtime-node#filesystem",
      service: "std/fs::FileSystem",
      operation: contract,
      entry: { close: () => ({ kind: "success", value: undefined }) },
      input: undefined,
      codecs: failureCodecs,
    })
    expect(synchronous.kind).toBe("defect")
    if (synchronous.kind === "defect") {
      expect(synchronous.defect.stage).toBe("call")
    }

    const malformed = await invokeProviderOperation({
      provider: "seseragi/runtime-node#filesystem",
      service: "std/fs::FileSystem",
      operation: contract,
      entry: { close: async () => ({ kind: "defect", defect: cause }) },
      input: undefined,
      codecs: failureCodecs,
    })
    expect(malformed.kind).toBe("defect")
    if (malformed.kind === "defect") {
      expect(malformed.defect.stage).toBe("result")
    }
  })

  test("wraps opaque resource tokens and enforces their owner and type", async () => {
    const handleType = {
      kind: "named",
      identity: "std/fs::FileHandle",
    } as const
    const open = operation(
      "std/fs::FileSystem#openRead",
      unit,
      handleType,
      never,
      "resource"
    )
    const token = { fd: 7 }
    const acquired = await invokeProviderOperation({
      provider: "seseragi/runtime-node#filesystem",
      service: "std/fs::FileSystem",
      operation: open,
      entry: {
        async openRead() {
          return { kind: "success", value: token }
        },
      },
      input: undefined,
      codecs,
    })
    expect(acquired.kind).toBe("success")
    if (acquired.kind !== "success") return
    expect(JSON.stringify(acquired.value)).toBe("{}")
    expect(acquired.value).not.toBe(token)

    let receivedToken: unknown
    const read = operation("std/fs::FileSystem#read", handleType, bytes)
    const readResult = await invokeProviderOperation({
      provider: "seseragi/runtime-node#filesystem",
      service: "std/fs::FileSystem",
      operation: read,
      entry: {
        async read(value: unknown) {
          receivedToken = value
          return { kind: "success", value: new Uint8Array([1, 2]) }
        },
      },
      input: acquired.value,
      codecs,
    })
    expect(readResult.kind).toBe("success")
    expect(receivedToken).toBe(token)

    for (const [provider, service, type] of [
      ["other/provider#filesystem", "std/fs::FileSystem", handleType],
      ["seseragi/runtime-node#filesystem", "std/other::FileSystem", handleType],
      [
        "seseragi/runtime-node#filesystem",
        "std/fs::FileSystem",
        { kind: "named", identity: "std/fs::OtherHandle" } as const,
      ],
    ] as const) {
      const rejected = await invokeProviderOperation({
        provider,
        service,
        operation: operation("std/fs::FileSystem#read", type, bytes),
        entry: { read: async () => ({ kind: "success", value: bytes }) },
        input: acquired.value,
        codecs,
      })
      expect(rejected.kind).toBe("defect")
      if (rejected.kind === "defect") {
        expect(rejected.defect.stage).toBe("input")
      }
    }
  })
})

import {
  assertProviderRuntimeAbi,
  decodeProviderValue,
  invokeProviderOperation,
  ProviderCodecRegistry,
  providerRuntimeAbi,
} from "@seseragi/runtime/provider"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

assertProviderRuntimeAbi({ ...providerRuntimeAbi })
const codecs = new ProviderCodecRegistry()
const bytes = { kind: "primitive", name: "bytes" } as const
const inputBytes = new Uint8Array([1, 2])
const outputBytes = decodeProviderValue(bytes, inputBytes, codecs) as Uint8Array
inputBytes[0] = 9
assert(outputBytes[0] === 1, "Bytes must cross the boundary as a snapshot")

let getterRead = false
const record = Object.defineProperty({}, "value", {
  get() {
    getterRead = true
    return 1
  },
})
try {
  decodeProviderValue(
    {
      kind: "record",
      fields: [{ name: "value", type: { kind: "primitive", name: "int" } }],
    },
    record,
    codecs
  )
} catch {
  // Accessor rejection is the expected boundary result.
}
assert(!getterRead, "record validation must not invoke provider getters")

const handleType = {
  kind: "named",
  identity: "std/fs::FileHandle",
} as const
const token = { fd: 7 }
const acquired = await invokeProviderOperation({
  provider: "seseragi/runtime-node#filesystem",
  service: "std/fs::FileSystem",
  operation: {
    identity: "std/fs::FileSystem#openRead",
    kind: "resource",
    input: { kind: "unit" },
    success: handleType,
    failure: { kind: "never" },
  },
  entry: {
    async openRead() {
      return { kind: "success", value: token }
    },
  },
  input: undefined,
  codecs,
})
assert(acquired.kind === "success", "resource acquisition must succeed")

let received: unknown
const read = await invokeProviderOperation({
  provider: "seseragi/runtime-node#filesystem",
  service: "std/fs::FileSystem",
  operation: {
    identity: "std/fs::FileSystem#read",
    kind: "one-shot",
    input: handleType,
    success: bytes,
    failure: { kind: "never" },
  },
  entry: {
    async read(value: unknown) {
      received = value
      return { kind: "success", value: new Uint8Array([3]) }
    },
  },
  input: acquired.value,
  codecs,
})
assert(read.kind === "success", "handle operation must succeed")
assert(received === token, "only the owning provider receives the host token")

const rejected = await invokeProviderOperation({
  provider: "other/provider#filesystem",
  service: "std/fs::FileSystem",
  operation: {
    identity: "std/fs::FileSystem#read",
    kind: "one-shot",
    input: handleType,
    success: bytes,
    failure: { kind: "never" },
  },
  entry: { read: async () => ({ kind: "success", value: new Uint8Array() }) },
  input: acquired.value,
  codecs,
})
assert(
  rejected.kind === "defect" && rejected.defect.stage === "input",
  "foreign provider handles must be rejected before the call"
)

process.stdout.write("provider runtime ABI probe passed\n")

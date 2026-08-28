import type { Bytes } from "./bytes"
import type { EffectContext } from "./effect"
import {
  bytesFromProvider,
  EmptyRandomIntRange,
  InvalidProbability,
  type Random,
  type RandomRangeError,
} from "./random"
import {
  invokeProviderOperation,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderLogicalType,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"
import { serviceFailure, serviceSuccess } from "./service"

const unit = Object.freeze({ kind: "unit" } as const)
const never = Object.freeze({ kind: "never" } as const)
const bool = Object.freeze({ kind: "primitive", name: "bool" } as const)
const bytes = Object.freeze({ kind: "primitive", name: "bytes" } as const)
const float = Object.freeze({ kind: "primitive", name: "float" } as const)
const int = Object.freeze({ kind: "primitive", name: "int" } as const)
const string = Object.freeze({ kind: "primitive", name: "string" } as const)
const rangeFailure = Object.freeze({
  kind: "named",
  identity: "std/random::RandomRangeError",
} as const)
const intArray = Object.freeze({ kind: "array", items: int } as const)

const contract = (
  name: string,
  input: ProviderLogicalType,
  success: ProviderLogicalType,
  failure: ProviderLogicalType = never
): ProviderOperationContract =>
  Object.freeze({
    identity: `std/random::Random#${name}`,
    kind: "one-shot",
    input,
    success,
    failure,
  })

const operations = Object.freeze({
  algorithmId: contract("algorithmId", unit, string),
  nextBool: contract("nextBool", unit, bool),
  nextInt: contract("nextInt", unit, int),
  intBetween: contract(
    "intBetween",
    {
      kind: "record",
      fields: [
        { name: "lower", type: int },
        { name: "upperExclusive", type: int },
      ],
    },
    int,
    rangeFailure
  ),
  unitFloat: contract("unitFloat", unit, float),
  chance: contract("chance", float, bool, rangeFailure),
  randomBytes: contract("randomBytes", int, bytes),
  chooseIndex: contract("chooseIndex", int, int),
  shuffleIndices: contract("shuffleIndices", int, intArray),
})

const codecs = new ProviderCodecRegistry([
  {
    identity: rangeFailure.identity,
    encode: (value) => value,
    decode: (value) => decodeRangeFailure(value),
  },
])

export function createProviderRandom(loaded: LoadedProviderEntry): Random {
  if (loaded.service !== "std/random::Random") {
    throw new TypeError(
      "resolved provider does not implement std/random::Random"
    )
  }
  const invoke = (
    operation: ProviderOperationContract,
    input: unknown,
    context: EffectContext
  ) =>
    invokeProviderOperation({
      provider: loaded.provider,
      service: loaded.service,
      operation,
      entry: loaded.entry,
      input,
      codecs,
      context,
    })
  return Object.freeze({
    async algorithmId(context) {
      return success(
        await invoke(operations.algorithmId, undefined, context)
      ) as string
    },
    async nextBool(context) {
      return success(
        await invoke(operations.nextBool, undefined, context)
      ) as boolean
    },
    async nextInt(context) {
      return success(
        await invoke(operations.nextInt, undefined, context)
      ) as number
    },
    async intBetween(lower, upperExclusive, context) {
      const outcome = await invoke(
        operations.intBetween,
        { lower, upperExclusive },
        context
      )
      return outcome.kind === "failure"
        ? serviceFailure(outcome.failure as RandomRangeError)
        : serviceSuccess(success(outcome) as number)
    },
    async unitFloat(context) {
      return success(
        await invoke(operations.unitFloat, undefined, context)
      ) as number
    },
    async chance(probability, context) {
      const outcome = await invoke(operations.chance, probability, context)
      return outcome.kind === "failure"
        ? serviceFailure(outcome.failure as RandomRangeError)
        : serviceSuccess(success(outcome) as boolean)
    },
    async randomBytes(size, context): Promise<Bytes> {
      return bytesFromProvider(
        success(await invoke(operations.randomBytes, size, context))
      )
    },
    async chooseIndex(length, context) {
      return success(
        await invoke(operations.chooseIndex, length, context)
      ) as number
    },
    async shuffleIndices(length, context) {
      return success(
        await invoke(operations.shuffleIndices, length, context)
      ) as ReadonlyArray<number>
    },
  })
}

function success(outcome: ProviderBridgeOutcome): unknown {
  if (outcome.kind === "defect") throw outcome.defect
  if (outcome.kind === "failure") {
    throw new TypeError("Random provider returned an unexpected typed failure")
  }
  return outcome.value
}

function decodeRangeFailure(value: unknown): RandomRangeError {
  if (typeof value !== "object" || value === null || !("tag" in value)) {
    throw new TypeError("RandomRangeError ABI value must be a tagged record")
  }
  const tagged = value as { tag: unknown; value?: unknown }
  if (tagged.tag === "InvalidProbability" && typeof tagged.value === "number") {
    return InvalidProbability(tagged.value)
  }
  if (
    tagged.tag === "EmptyRandomIntRange" &&
    typeof tagged.value === "object" &&
    tagged.value !== null &&
    "lower" in tagged.value &&
    "upperExclusive" in tagged.value
  ) {
    const range = tagged.value as { lower: unknown; upperExclusive: unknown }
    if (
      typeof range.lower === "number" &&
      typeof range.upperExclusive === "number"
    ) {
      return EmptyRandomIntRange({
        lower: range.lower,
        upperExclusive: range.upperExclusive,
      })
    }
  }
  throw new TypeError("RandomRangeError ABI value is invalid")
}

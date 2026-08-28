import type { Bytes } from "./bytes"
import {
  bytesFromProvider,
  EntropyReadFailure,
  type Entropy,
  type EntropyError,
  EntropyUnavailable,
} from "./entropy"
import type { EffectContext } from "./effect"
import {
  invokeProviderOperation,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"
import { serviceFailure, serviceSuccess } from "./service"

const errorType = Object.freeze({
  kind: "named",
  identity: "std/entropy::EntropyError",
} as const)
const secureBytesContract: ProviderOperationContract = Object.freeze({
  identity: "std/entropy::Entropy#secureBytes",
  kind: "one-shot",
  input: { kind: "primitive", name: "int" } as const,
  success: { kind: "primitive", name: "bytes" } as const,
  failure: errorType,
})
const codecs = new ProviderCodecRegistry([
  {
    identity: errorType.identity,
    encode: (value) => value,
    decode: (value) => {
      if (isTag(value, "EntropyUnavailable")) return EntropyUnavailable
      if (isTag(value, "EntropyReadFailure")) return EntropyReadFailure
      throw new TypeError("EntropyError ABI value is invalid")
    },
  },
])

export function createProviderEntropy(loaded: LoadedProviderEntry): Entropy {
  if (loaded.service !== "std/entropy::Entropy") {
    throw new TypeError(
      "resolved provider does not implement std/entropy::Entropy"
    )
  }
  return Object.freeze({
    async secureBytes(size: number, context: EffectContext) {
      const outcome = await invokeProviderOperation({
        provider: loaded.provider,
        service: loaded.service,
        operation: secureBytesContract,
        entry: loaded.entry,
        input: size,
        codecs,
        context,
      })
      return outcome.kind === "failure"
        ? serviceFailure(outcome.failure as EntropyError)
        : serviceSuccess(bytesFromProvider(success(outcome)) as Bytes)
    },
  })
}

function success(outcome: ProviderBridgeOutcome): unknown {
  if (outcome.kind === "defect") throw outcome.defect
  if (outcome.kind === "failure") {
    throw new TypeError("Entropy provider returned an unexpected typed failure")
  }
  return outcome.value
}

function isTag(value: unknown, tag: string): boolean {
  return (
    typeof value === "object" &&
    value !== null &&
    "tag" in value &&
    (value as { tag: unknown }).tag === tag
  )
}

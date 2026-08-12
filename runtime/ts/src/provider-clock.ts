import type { Clock } from "./clock"
import {
  createInstant,
  type Duration,
  durationNanoseconds,
  type Instant,
} from "./clock-value"
import type { EffectContext, Unit } from "./effect"
import {
  invokeProviderOperation,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"

const unit = Object.freeze({ kind: "unit" } as const)
const never = Object.freeze({ kind: "never" } as const)
const duration = Object.freeze({
  kind: "named",
  identity: "std/time::Duration",
} as const)
const instant = Object.freeze({
  kind: "named",
  identity: "std/time::Instant",
} as const)

const nowContract: ProviderOperationContract = Object.freeze({
  identity: "std/clock::Clock#now",
  kind: "one-shot",
  input: unit,
  success: instant,
  failure: never,
})

const sleepContract: ProviderOperationContract = Object.freeze({
  identity: "std/clock::Clock#sleep",
  kind: "one-shot",
  input: duration,
  success: unit,
  failure: never,
})

const clockCodecs = new ProviderCodecRegistry([
  {
    identity: duration.identity,
    encode: (value) => durationNanoseconds(value as Duration),
    decode: (value) => {
      if (typeof value !== "bigint") {
        throw new TypeError("Duration ABI value must be a bigint")
      }
      return value
    },
  },
  {
    identity: instant.identity,
    encode: (value) => value,
    decode: (value) => {
      if (typeof value !== "bigint") {
        throw new TypeError("Instant ABI value must be a bigint")
      }
      return createInstant(value)
    },
  },
])

/** Builds the app-facing Clock service from one resolved provider entry. */
export function createProviderClock(loaded: LoadedProviderEntry): Clock {
  if (loaded.service !== "std/clock::Clock") {
    throw new TypeError("resolved provider does not implement std/clock::Clock")
  }
  return Object.freeze({
    async now(context: EffectContext): Promise<Instant> {
      const outcome = await invokeProviderOperation({
        provider: loaded.provider,
        service: loaded.service,
        operation: nowContract,
        entry: loaded.entry,
        input: undefined,
        codecs: clockCodecs,
        context,
      })
      return successValue(outcome) as Instant
    },
    async sleep(value: Duration, context: EffectContext): Promise<Unit> {
      const outcome = await invokeProviderOperation({
        provider: loaded.provider,
        service: loaded.service,
        operation: sleepContract,
        entry: loaded.entry,
        input: value,
        codecs: clockCodecs,
        context,
      })
      return successValue(outcome) as Unit
    },
  })
}

function successValue(outcome: ProviderBridgeOutcome): unknown {
  if (outcome.kind === "defect") throw outcome.defect
  if (outcome.kind === "failure") {
    throw new TypeError("Clock provider returned an impossible typed failure")
  }
  return outcome.value
}

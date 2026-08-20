import type { EffectContext } from "./effect"
import {
  type Location,
  locationFromHref,
  type Navigation,
  type NavigationError,
  navigationFailure,
  navigationSuccess,
  type Url,
} from "./navigation"
import {
  invokeProviderOperation,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"

const unit = Object.freeze({ kind: "unit" } as const)
const never = Object.freeze({ kind: "never" } as const)
const url = Object.freeze({
  kind: "named",
  identity: "std/web/navigation::Url",
} as const)
const location = Object.freeze({
  kind: "named",
  identity: "std/web/navigation::Location",
} as const)
const navigationError = Object.freeze({
  kind: "named",
  identity: "std/web/navigation::NavigationError",
} as const)

const contract = (
  name: string,
  input: ProviderOperationContract["input"],
  success: ProviderOperationContract["success"],
  failure: ProviderOperationContract["failure"] = navigationError
): ProviderOperationContract =>
  Object.freeze({
    identity: `std/web/navigation::Navigation#${name}`,
    kind: "one-shot",
    input,
    success,
    failure,
  })

const currentContract = contract("current", unit, location)
const pushContract = contract("push", url, location)
const replaceContract = contract("replace", url, location)
const backContract = contract("back", unit, unit)
const forwardContract = contract("forward", unit, unit)
const nextChangeContract = contract("nextChange", unit, location, never)

const codecs = new ProviderCodecRegistry([
  {
    identity: url.identity,
    encode: (value) => (value as Url).href,
    decode: (value) => {
      if (typeof value !== "string") {
        throw new TypeError("navigation URL ABI value must be a string")
      }
      return value
    },
  },
  {
    identity: location.identity,
    encode: (value) => (value as Location).url.href,
    decode: (value) => {
      if (typeof value !== "string") {
        throw new TypeError("navigation location ABI value must be a string")
      }
      return locationFromHref(value)
    },
  },
  {
    identity: navigationError.identity,
    encode: (value) => value,
    decode: (value) => decodeNavigationError(value),
  },
])

export function createProviderNavigation(
  loaded: LoadedProviderEntry
): Navigation {
  if (loaded.service !== "std/web/navigation::Navigation") {
    throw new TypeError(
      "resolved provider does not implement std/web/navigation::Navigation"
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
    async current(context) {
      return outcome(await invoke(currentContract, undefined, context))
    },
    async push(value, context) {
      return outcome(await invoke(pushContract, value, context))
    },
    async replace(value, context) {
      return outcome(await invoke(replaceContract, value, context))
    },
    async back(context) {
      return outcome(await invoke(backContract, undefined, context))
    },
    async forward(context) {
      return outcome(await invoke(forwardContract, undefined, context))
    },
    async nextChange(context) {
      const result = await invoke(nextChangeContract, undefined, context)
      if (result.kind === "defect") throw result.defect
      if (result.kind === "failure") {
        throw new TypeError(
          "navigation nextChange provider returned an impossible typed failure"
        )
      }
      return result.value as Location
    },
  })
}

function outcome<Success>(value: ProviderBridgeOutcome) {
  if (value.kind === "defect") throw value.defect
  return value.kind === "failure"
    ? navigationFailure(value.failure as NavigationError)
    : navigationSuccess(value.value as Success)
}

function decodeNavigationError(value: unknown): NavigationError {
  if (typeof value !== "object" || value === null || !("tag" in value)) {
    throw new TypeError("navigation error ABI value is invalid")
  }
  const error = value as { tag?: unknown; value?: unknown }
  if (
    (error.tag === "NavigationUnavailable" ||
      error.tag === "NavigationSecurityFailure") &&
    typeof error.value === "string"
  ) {
    return Object.freeze({ tag: error.tag, value: error.value })
  }
  if (
    error.tag === "CrossOriginNavigation" &&
    typeof error.value === "object" &&
    error.value !== null &&
    "expected" in error.value &&
    "actual" in error.value &&
    typeof error.value.expected === "string" &&
    typeof error.value.actual === "string"
  ) {
    return Object.freeze({
      tag: error.tag,
      value: Object.freeze({
        expected: error.value.expected,
        actual: error.value.actual,
      }),
    })
  }
  throw new TypeError("navigation error ABI value is invalid")
}

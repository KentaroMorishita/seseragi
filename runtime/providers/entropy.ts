import {
  providerRuntimeAbi,
  type ProviderResult,
} from "@seseragi/runtime/provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
  type ProviderRuntimeTarget,
} from "@seseragi/runtime/provider-package"

export type EntropyHost = Readonly<{
  available?: () => boolean
  fill: (values: Uint8Array) => void
}>

export function createEntropyProvider(
  identity: string,
  targets: readonly ProviderRuntimeTarget[],
  host: EntropyHost
): ProviderPackageEntry {
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: identity,
    service: "std/entropy::Entropy",
    targets,
    operations: {
      async secureBytes(value) {
        if (typeof value !== "number" || !Number.isSafeInteger(value)) {
          throw new TypeError("Entropy byte size must be a safe integer")
        }
        if (host.available?.() === false) {
          return failure({ tag: "EntropyUnavailable" })
        }
        const bytes = new Uint8Array(value)
        try {
          host.fill(bytes)
          return success(bytes)
        } catch {
          return failure({ tag: "EntropyReadFailure" })
        }
      },
    },
  })
}

function success(value: unknown): ProviderResult {
  return { kind: "success", value }
}

function failure(value: unknown): ProviderResult {
  return { kind: "failure", failure: value }
}

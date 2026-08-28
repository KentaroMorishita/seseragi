import {
  providerRuntimeAbi,
  type ProviderResult,
} from "@seseragi/runtime/provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
} from "@seseragi/runtime/provider-package"

const MASK = (1n << 64n) - 1n
const TWO_64 = 1n << 64n
const TWO_53 = 1n << 53n
const TWO_52 = 1n << 52n
const ALGORITHM_ID = "seseragi-xoshiro256ss-v1"

export type RandomSeedSource = () => bigint

export function createRandomProvider(
  seedSource: RandomSeedSource
): ProviderPackageEntry {
  let state: [bigint, bigint, bigint, bigint] | undefined
  const current = (): [bigint, bigint, bigint, bigint] => {
    state ??= expandSeed(seedSource())
    return state
  }
  const next = (): bigint => nextOutput(current())
  const sampleIndex = (length: number): number => {
    if (!Number.isSafeInteger(length) || length <= 0) {
      throw new TypeError("Random sample length must be a positive safe integer")
    }
    return Number(sampleWidth(BigInt(length), next))
  }
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: "seseragi/runtime#random",
    service: "std/random::Random",
    targets: ["bun-process", "browser"],
    operations: {
      async algorithmId() {
        return success(ALGORITHM_ID)
      },
      async nextBool() {
        return success((next() & 1n) === 1n)
      },
      async nextInt() {
        const value = next() >> 11n
        return success(Number(value >= TWO_52 ? value - TWO_53 : value))
      },
      async intBetween(value) {
        const range = record(value)
        const lower = safeInt(range.lower)
        const upperExclusive = safeInt(range.upperExclusive)
        if (lower >= upperExclusive) {
          return failure({
            tag: "EmptyRandomIntRange",
            value: { lower, upperExclusive },
          })
        }
        const width = BigInt(upperExclusive) - BigInt(lower)
        return success(Number(BigInt(lower) + sampleWidth(width, next)))
      },
      async unitFloat() {
        return success(Number(next() >> 11n) / 2 ** 53)
      },
      async chance(value) {
        if (
          typeof value !== "number" ||
          Number.isNaN(value) ||
          value < 0 ||
          value > 1
        ) {
          return failure({ tag: "InvalidProbability", value })
        }
        if (value === 0) return success(false)
        if (value === 1) return success(true)
        return success(Number(next() >> 11n) / 2 ** 53 < value)
      },
      async randomBytes(value) {
        const size = safeInt(value)
        if (size <= 0 || size > 1024 * 1024) {
          throw new TypeError("Random byte size is outside the standard range")
        }
        const bytes = new Uint8Array(size)
        let offset = 0
        while (offset < bytes.length) {
          let output = next()
          for (let index = 0; index < 8 && offset < bytes.length; index += 1) {
            bytes[offset] = Number(output & 0xffn)
            output >>= 8n
            offset += 1
          }
        }
        return success(bytes)
      },
      async chooseIndex(value) {
        return success(sampleIndex(safeInt(value)))
      },
      async shuffleIndices(value) {
        const length = safeInt(value)
        if (length < 0) throw new TypeError("Random shuffle length must be non-negative")
        const indices = Array.from({ length }, (_, index) => index)
        for (let index = indices.length - 1; index > 0; index -= 1) {
          const swap = sampleIndex(index + 1)
          const valueAtIndex = indices[index] as number
          indices[index] = indices[swap] as number
          indices[swap] = valueAtIndex
        }
        return success(indices)
      },
    },
  })
}

function expandSeed(seed: bigint): [bigint, bigint, bigint, bigint] {
  let state = BigInt.asUintN(64, seed)
  const next = (): bigint => {
    state = (state + 0x9e3779b97f4a7c15n) & MASK
    let value = state
    value = ((value ^ (value >> 30n)) * 0xbf58476d1ce4e5b9n) & MASK
    value = ((value ^ (value >> 27n)) * 0x94d049bb133111ebn) & MASK
    return (value ^ (value >> 31n)) & MASK
  }
  return [next(), next(), next(), next()]
}

function nextOutput(state: [bigint, bigint, bigint, bigint]): bigint {
  const output = (rotateLeft((state[1] * 5n) & MASK, 7n) * 9n) & MASK
  const shifted = (state[1] << 17n) & MASK
  state[2] ^= state[0]
  state[3] ^= state[1]
  state[1] ^= state[2]
  state[0] ^= state[3]
  state[2] ^= shifted
  state[3] = rotateLeft(state[3], 45n)
  return output
}

function rotateLeft(value: bigint, shift: bigint): bigint {
  return ((value << shift) | (value >> (64n - shift))) & MASK
}

function sampleWidth(width: bigint, next: () => bigint): bigint {
  const limit = TWO_64 - (TWO_64 % width)
  while (true) {
    const output = next()
    if (output < limit) return output % width
  }
}

function safeInt(value: unknown): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value)) {
    throw new TypeError("Random Int ABI value must be a safe integer")
  }
  return value
}

function record(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new TypeError("Random record ABI value is invalid")
  }
  return value as Record<string, unknown>
}

function success(value: unknown): ProviderResult {
  return { kind: "success", value }
}

function failure(value: unknown): ProviderResult {
  return { kind: "failure", failure: value }
}

import { createRandomProvider } from "../random"

declare global {
  var __SESERAGI_RANDOM_SEED__: string | undefined
}

function seed(): bigint {
  if (globalThis.__SESERAGI_RANDOM_SEED__ !== undefined) {
    return BigInt(globalThis.__SESERAGI_RANDOM_SEED__)
  }
  const values = new BigUint64Array(1)
  globalThis.crypto.getRandomValues(values)
  return values[0] as bigint
}

export const provider = createRandomProvider(seed)

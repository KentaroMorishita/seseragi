import { createRandomProvider } from "../random"

function seed(): bigint {
  const configured = process.env.SESERAGI_RANDOM_SEED
  if (configured !== undefined) return BigInt(configured)
  const values = new BigUint64Array(1)
  globalThis.crypto.getRandomValues(values)
  return values[0] as bigint
}

export const provider = createRandomProvider(seed)

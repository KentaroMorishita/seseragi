import { canonicalExample } from "./canonical-example"

const directory = "apps/site/examples/comparisons/shipping-total"

export const entranceComparison = {
  id: "shipping-total",
  seseragi: `${directory}/seseragi.ssrg`,
  typescript: `${directory}/typescript.ts`,
  expectedOutput: `${directory}/expected.stdout`,
} as const

// Author-facing evidence, not a second page-content format. Both code panels
// use the same ExampleSource pipeline as the rest of the site.
export function comparisonExamples(playgroundUrl: string) {
  return [
    canonicalExample(
      "comparison-shipping-total-seseragi",
      entranceComparison.seseragi,
      playgroundUrl
    ),
    canonicalExample(
      "comparison-shipping-total-typescript",
      entranceComparison.typescript,
      playgroundUrl,
      false,
      "typescript"
    ),
  ]
}

// All amounts are integer yen. These cases intentionally stay inside the
// shared 0..1,000,000 input domain explained in the comparison decision.
export const shippingCases = [
  { freeFrom: 5000, fee: 500, subtotal: 0, expected: 500 },
  { freeFrom: 5000, fee: 500, subtotal: 3200, expected: 3700 },
  { freeFrom: 5000, fee: 500, subtotal: 4999, expected: 5499 },
  { freeFrom: 5000, fee: 500, subtotal: 5000, expected: 5000 },
  { freeFrom: 5000, fee: 500, subtotal: 5001, expected: 5001 },
  { freeFrom: 5000, fee: 500, subtotal: 7800, expected: 7800 },
  { freeFrom: 0, fee: 500, subtotal: 0, expected: 0 },
  { freeFrom: 5000, fee: 0, subtotal: 3200, expected: 3200 },
  { freeFrom: 8000, fee: 700, subtotal: 5000, expected: 5700 },
  {
    freeFrom: 1000000,
    fee: 1000000,
    subtotal: 999999,
    expected: 1999999,
  },
] as const

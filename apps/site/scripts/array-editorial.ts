import { canonicalExample } from "./canonical-example"

// IDs and expected execution results only; editorial prose belongs to typed locale modules.
export const arrayEditorialCases = [
  {
    name: "chunksOf",
    slug: "chunksof",
    identity: "std/array::chunksOf",
    expected:
      "Right [[10, 20], [30, 40], [50]]\nRight []\nLeft NonPositiveSize 0\nLeft NonPositiveSize -2",
  },
  {
    name: "windows",
    slug: "windows",
    identity: "std/array::windows",
    expected:
      "Right [[10, 20], [20, 30], [30, 40]]\nRight []\nRight []\nLeft NonPositiveSize 0",
  },
  {
    name: "dropWhile",
    slug: "dropwhile",
    identity: "std/array::dropWhile",
    expected: "[4, 1]\n[4, 1]\n[]\n[]",
  },
  {
    name: "takeWhile",
    slug: "takewhile",
    identity: "std/array::takeWhile",
    expected: "[1, 2]\n[]\n[1, 2]\n[]",
  },
  {
    name: "findIndex",
    slug: "findindex",
    identity: "std/array::findIndex",
    expected: "found at 1\nnot found\nnot found",
  },
  {
    name: "init",
    slug: "init",
    identity: "std/array::init",
    expected: "Just [10, 20]\nJust []\nNothing",
  },
  {
    name: "last",
    slug: "last",
    identity: "std/array::last",
    expected: "Just 30\nJust 10\nNothing",
  },
  {
    name: "reduceRight",
    slug: "reduceright",
    identity: "std/array::reduceRight",
    expected: "2\n10",
  },
  {
    name: "sort",
    slug: "sort",
    identity: "std/array::sort",
    expected: "[2, 2, 10, 30]\n[30, 2, 10, 2]\n[]",
  },
  {
    name: "sortBy",
    slug: "sortby",
    identity: "std/array::sortBy",
    expected:
      "[{ name: Mio, score: 1 }, { name: Aki, score: 2 }, { name: Ren, score: 2 }]\n[{ name: Aki, score: 2 }, { name: Mio, score: 1 }, { name: Ren, score: 2 }]\n[]",
  },
  {
    name: "groupBy",
    slug: "groupby",
    identity: "std/array::groupBy",
    expected: "[1, 0]\nJust [3, 5, 7]\nJust [2, 4]\n0",
  },
] as const

export const arrayConsumerCases = [
  { name: "consumeMaybe", slug: "consume-maybe", expected: "last: 30\nempty" },
  {
    name: "consumeEither",
    slug: "consume-either",
    expected: "batches: [[10, 20], [30]]\ninvalid: NonPositiveSize 0",
  },
] as const

export function arrayEditorialExamples(playgroundUrl: string) {
  return [...arrayEditorialCases, ...arrayConsumerCases].map(({ slug }) =>
    canonicalExample(
      `array-${slug}`,
      `apps/site/examples/src/api-array/${slug}.ssrg`,
      playgroundUrl
    )
  )
}

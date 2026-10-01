import { canonicalExample } from "./canonical-example"

export const listEditorialCases = [
  {
    name: "chunksOf",
    slug: "chunksof",
    identity: "std/list::chunksOf",
    expected:
      "batches: `[`[10, 20], `[30, 40], `[50]]\nbatches: `[`[10, 20]]\nbatches: `[`[10], `[20]]\nbatches: `[]\ninvalid: NonPositiveSize 0\ninvalid: NonPositiveSize -2",
  },
  {
    name: "windows",
    slug: "windows",
    identity: "std/list::windows",
    expected:
      "windows: `[`[10, 20], `[20, 30], `[30, 40]]\nwindows: `[]\nwindows: `[`[10], `[20]]\nwindows: `[]\ninvalid: NonPositiveSize 0\ninvalid: NonPositiveSize -2",
  },
  {
    name: "findIndex",
    slug: "findindex",
    identity: "std/list::findIndex",
    expected: "found at 0\nfound at 1\nnot found\nnot found",
  },
  {
    name: "init",
    slug: "init",
    identity: "std/list::init",
    expected: "remaining: `[10, 20]\nremaining: `[]\nempty input",
  },
  {
    name: "last",
    slug: "last",
    identity: "std/list::last",
    expected: "last: 30\nlast: 10\nempty input",
  },
  {
    name: "reduceRight",
    slug: "reduceright",
    identity: "std/list::reduceRight",
    expected: "2\n10",
  },
  {
    name: "sort",
    slug: "sort",
    identity: "std/list::sort",
    expected: "`[2, 2, 10, 30]\n`[30, 2, 10, 2]\n`[]",
  },
  {
    name: "sortBy",
    slug: "sortby",
    identity: "std/list::sortBy",
    expected:
      "`[{ name: Mio, score: 1 }, { name: Aki, score: 2 }, { name: Ren, score: 2 }]\n`[{ name: Aki, score: 2 }, { name: Mio, score: 1 }, { name: Ren, score: 2 }]\n`[]",
  },
  {
    name: "groupBy",
    slug: "groupby",
    identity: "std/list::groupBy",
    expected: "[1, 0]\ngroup: `[3, 5, 7]\ngroup: `[2, 4]\nno group\n0",
  },
  {
    name: "takeWhile",
    slug: "takewhile",
    identity: "std/list::takeWhile",
    expected: "`[1, 2]\n`[]\n`[1, 2]\n`[]",
  },
  {
    name: "dropWhile",
    slug: "dropwhile",
    identity: "std/list::dropWhile",
    expected: "`[0, 3]\n`[0, 3]\n`[]\n`[]",
  },
] as const

export const listConsumerCases = [
  {
    name: "consume-maybe",
    slug: "consume-maybe",
    expected: "last: 30\nlast: 10\nempty input",
  },
  {
    name: "consume-either",
    slug: "consume-either",
    expected:
      "batches: `[`[10, 20], `[30, 40], `[50]]\nbatches: `[`[10, 20]]\nbatches: `[`[10], `[20]]\nbatches: `[]\ninvalid: NonPositiveSize 0\ninvalid: NonPositiveSize -2",
  },
] as const

export function listEditorialExamples(playgroundUrl: string) {
  return [...listEditorialCases, ...listConsumerCases].map(({ slug }) =>
    canonicalExample(
      `list-${slug}`,
      `apps/site/examples/src/api-list/${slug}.ssrg`,
      playgroundUrl
    )
  )
}

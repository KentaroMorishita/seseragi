import { canonicalExample } from "./canonical-example"

export const nonemptyIteratorReaderCases = [
  {
    slug: "nonempty-module",
    module: "std/non-empty-list",
    name: "",
    identity: "std/non-empty-list",
    output: "60\nno scores\n",
    typescriptOutput: "60\nno scores\n",
  },
  {
    slug: "singleton",
    module: "std/non-empty-list",
    name: "singleton",
    identity: "std/non-empty-list::singleton",
    output: "10\n`[10]\n`[]\n",
    typescriptOutput: "10\n[10]\n[]\n",
  },
  {
    slug: "cons",
    module: "std/non-empty-list",
    name: "cons",
    identity: "std/non-empty-list::cons",
    output: "`[10, 20, 30]\n`[20, 30]\n`[10]\n",
    typescriptOutput: "[10,20,30]\n[20,30]\n[10]\n",
  },
  {
    slug: "fromlist",
    module: "std/non-empty-list",
    name: "fromList",
    identity: "std/non-empty-list::fromList",
    output: "first: 0\nno score\n",
    typescriptOutput: "first: 0\nno score\n",
  },
  {
    slug: "reduce1",
    module: "std/non-empty-list",
    name: "reduce1",
    identity: "std/non-empty-list::reduce1",
    output: "15\n20\n`[20, 3, 2]\n",
    typescriptOutput: "15\n20\n[20,3,2]\n",
  },
  {
    slug: "iterator-module",
    module: "std/iterator",
    name: "",
    identity: "std/iterator",
    output: "[3, 2, 1]\n[3, 2, 1]\n[]\n",
    typescriptOutput: "[3,2,1]\n[3,2,1]\n[]\n",
  },
  {
    slug: "next",
    module: "std/iterator",
    name: "next",
    identity: "std/iterator::next",
    output: "3\n3\n2\ndone\n",
    typescriptOutput: "3\n2\n1\ndone\n",
  },
  {
    slug: "unfold",
    module: "std/iterator",
    name: "unfold",
    identity: "std/iterator::unfold",
    output: "[3, 2, 1]\n[]\n10\n",
    typescriptOutput: "[3,2,1]\n[]\n10\n",
  },
] as const

export function nonemptyIteratorReaderExamples(playgroundUrl: string) {
  return nonemptyIteratorReaderCases.flatMap(({ slug }) => [
    canonicalExample(
      `sequence-reader-${slug}`,
      `apps/site/examples/src/api-nonempty-iterator/${slug}.ssrg`,
      playgroundUrl
    ),
    canonicalExample(
      `sequence-reader-${slug}-ts`,
      `apps/site/examples/comparisons/api-nonempty-iterator/${slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}

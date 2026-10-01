import { canonicalExample } from "./canonical-example"

// Only identity, source selection, and expected output; prose belongs to typed locale files.
export const sequenceEditorialCases = [
  {
    module: "array",
    name: "empty",
    slug: "empty",
    identity: "std/array::empty",
    expected: "[]\n[]\n[]",
  },
  {
    module: "array",
    name: "singleton",
    slug: "singleton",
    identity: "std/array::singleton",
    expected: "[42]\n[Aki]\n[[]]\n1",
  },
  {
    module: "array",
    name: "fromIterable",
    slug: "fromiterable",
    identity: "std/array::fromIterable",
    expected: "[30, 10, 20]\n`[30, 10, 20]\n[]",
  },
  {
    module: "array",
    name: "zip",
    slug: "zip",
    identity: "std/array::zip",
    expected: "[(Aki, 80), (Mio, 95)]\n[(80, Aki), (95, Mio)]\n[]\n[]",
  },
  {
    module: "array",
    name: "zipWith",
    slug: "zipwith",
    identity: "std/array::zipWith",
    expected: "[9, 18]\n[-9, -18]\n[]\n[]",
  },
  {
    module: "array",
    name: "unzip",
    slug: "unzip",
    identity: "std/array::unzip",
    expected: "[Aki, Mio]\n[80, 95]\n([], [])",
  },
  {
    module: "list",
    name: "empty",
    slug: "empty",
    identity: "std/list::empty",
    expected: "`[]\n`[]\n`[]",
  },
  {
    module: "list",
    name: "singleton",
    slug: "singleton",
    identity: "std/list::singleton",
    expected: "`[42]\n`[Aki]\n`[`[]]\n1",
  },
  {
    module: "list",
    name: "fromIterable",
    slug: "fromiterable",
    identity: "std/list::fromIterable",
    expected: "`[30, 10, 20]\n[30, 10, 20]\n`[]",
  },
  {
    module: "list",
    name: "zip",
    slug: "zip",
    identity: "std/list::zip",
    expected: "`[(Aki, 80), (Mio, 95)]\n`[(80, Aki), (95, Mio)]\n`[]\n`[]",
  },
  {
    module: "list",
    name: "zipWith",
    slug: "zipwith",
    identity: "std/list::zipWith",
    expected: "`[9, 18]\n`[-9, -18]\n`[]\n`[]",
  },
  {
    module: "list",
    name: "unzip",
    slug: "unzip",
    identity: "std/list::unzip",
    expected: "`[Aki, Mio]\n`[80, 95]\n(`[], `[])",
  },
] as const

export function sequenceEditorialExamples(playgroundUrl: string) {
  return sequenceEditorialCases.map(({ module, slug }) =>
    canonicalExample(
      `sequence-${module}-${slug}`,
      `apps/site/examples/src/api-sequence/${module}/${slug}.ssrg`,
      playgroundUrl
    )
  )
}

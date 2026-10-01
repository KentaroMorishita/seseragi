import { canonicalExample } from "./canonical-example"

const outputs = {
  map: {
    output:
      "[(tea, 2), (coffee, 5)]\n[(tea, 3), (coffee, 5)]\n0\nnot listed\ntrue\n",
    typescriptOutput:
      '[["tea",2],["coffee",5]]\n[["tea",3],["coffee",5]]\n0\nnot listed\ntrue\n',
  },
  set: {
    output: "[work, home]\n[work, home, travel]\ntrue\n0\n",
    typescriptOutput: '["work","home"]\n["work","home","travel"]\ntrue\n0\n',
  },
  nonempty: {
    output: "10\n`[20, 30]\n`[]\n0\nno scores\n",
    typescriptOutput: "10\n[20,30]\n[]\n0\nno scores\n",
  },
  iterator: {
    output: "3\n3\n2\ndone\n",
    typescriptOutput: "3\n3\n2\ndone\n",
  },
  "size-error": {
    output:
      "[[10, 20], [30]]\n[]\nsize must be positive: 0\nsize must be positive: -2\nNonPositiveSize 2\n",
    typescriptOutput:
      '[[10,20],[30]]\n[]\nsize must be positive: 0\nsize must be positive: -2\n{"kind":"non-positive-size","size":2}\n',
  },
  "reduce-step": {
    output: "30\n30\n0\n99\nstill running\n",
    // The first total is the ordinary loop; the next three use the callback.
    typescriptOutput: "30\n30\n30\n0\n99\nstill running\n",
  },
} as const

export const collectionTypeCases = [
  {
    module: "std/map",
    name: "Map",
    identity: "std/map::Map",
    namespace: "type",
    itemKind: "opaque-type",
    signature: "Map<_, _>",
    slug: "map",
    nativeSlug: "map",
    ...outputs.map,
  },
  {
    module: "std/set",
    name: "Set",
    identity: "std/set::Set",
    namespace: "type",
    itemKind: "opaque-type",
    signature: "Set<_>",
    slug: "set",
    nativeSlug: "set",
    ...outputs.set,
  },
  {
    module: "std/non-empty-list",
    name: "NonEmptyList",
    identity: "std/non-empty-list::NonEmptyList",
    namespace: "type",
    itemKind: "opaque-type",
    signature: "NonEmptyList<_>",
    slug: "nonemptylist",
    nativeSlug: "nonempty",
    ...outputs.nonempty,
  },
  {
    module: "std/iterator",
    name: "Iterator",
    identity: "std/prelude::Iterator",
    namespace: "type",
    itemKind: "opaque-type",
    signature: "Iterator<_>",
    slug: "iterator",
    nativeSlug: "iterator",
    ...outputs.iterator,
  },
  {
    module: "std/collection",
    name: "SizeError",
    identity: "std/collection::SizeError",
    namespace: "type",
    itemKind: "opaque-type",
    signature: "SizeError",
    slug: "sizeerror",
    nativeSlug: "size-error",
    ...outputs["size-error"],
  },
  {
    module: "std/collection",
    name: "NonPositiveSize",
    identity: "std/collection::NonPositiveSize",
    namespace: "value",
    itemKind: "constructor",
    signature: "NonPositiveSize arg1: Int -> SizeError",
    slug: "nonpositivesize",
    nativeSlug: "size-error",
    ...outputs["size-error"],
  },
  {
    module: "std/collection",
    name: "ReduceStep",
    identity: "std/collection::ReduceStep",
    namespace: "type",
    itemKind: "opaque-type",
    signature: "ReduceStep<_>",
    slug: "reducestep",
    nativeSlug: "reduce-step",
    ...outputs["reduce-step"],
  },
  {
    module: "std/collection",
    name: "Next",
    identity: "std/collection::Next",
    namespace: "value",
    itemKind: "constructor",
    signature: "Next<A> arg1: A -> ReduceStep<A>",
    slug: "next",
    nativeSlug: "reduce-step",
    ...outputs["reduce-step"],
  },
  {
    module: "std/collection",
    name: "Done",
    identity: "std/collection::Done",
    namespace: "value",
    itemKind: "constructor",
    signature: "Done<A> arg1: A -> ReduceStep<A>",
    slug: "done",
    nativeSlug: "reduce-step",
    ...outputs["reduce-step"],
  },
] as const

export const collectionTypePrograms = collectionTypeCases.filter(
  (item, index, items) =>
    items.findIndex((other) => other.nativeSlug === item.nativeSlug) === index
)

export const collectionTypeInvalidCases = [
  "map-record",
  "set-array",
  "nonempty-empty-list",
  "nonempty-unchecked-list",
  "iterator-array",
  "size-error-int",
  "reduce-step-bare-int",
  "list-array",
] as const

export function collectionTypeExamples(playgroundUrl: string) {
  return [
    ...collectionTypePrograms.flatMap(({ nativeSlug }) => [
      canonicalExample(
        `collection-type-${nativeSlug}`,
        `apps/site/examples/src/collection-types/${nativeSlug}.ssrg`,
        playgroundUrl
      ),
      canonicalExample(
        `collection-type-${nativeSlug}-ts`,
        `apps/site/examples/comparisons/collection-types/${nativeSlug}.ts`,
        playgroundUrl,
        false,
        "typescript"
      ),
    ]),
    ...collectionTypeInvalidCases.map((slug) =>
      canonicalExample(
        `collection-type-invalid-${slug}`,
        `apps/site/examples/invalid/src/collection-types/${slug}.ssrg`,
        playgroundUrl,
        false
      )
    ),
  ]
}

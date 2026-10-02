import { canonicalExample } from "./canonical-example"

export const collectionTransformCases = [
  {
    slug: "array-append",
    output:
      '["Read","Review","Ship"]\n["Read","Review","Ship"]\n["Read","Review"]\n["Ship"]\n',
  },
  {
    slug: "array-concat",
    output: '["Read","Review","Ship"]\n',
  },
  {
    slug: "array-filtermap",
    output: '[2,0,3]\n["2","oops","0","3"]\n',
  },
  {
    slug: "array-flatmap",
    output: '["Read","Review","Ship"]\n',
  },
  {
    slug: "array-reverse",
    output: '["Ship","Review","Read"]\n["Read","Review","Ship"]\n',
  },
  {
    slug: "array-tail",
    output: 'missing\npresent: []\npresent: ["Review","Ship"]\n',
  },
  {
    slug: "array-tolist",
    output: '["Read","Review","Ship"]\n["Read","Review","Ship"]\n',
  },
  {
    slug: "list-append",
    output:
      '["Read","Review","Ship"]\n["Read","Review","Ship"]\n["Read","Review"]\n["Ship"]\n',
  },
  {
    slug: "list-concat",
    output: '["Read","Review","Ship"]\n',
  },
  {
    slug: "list-filtermap",
    output: '[2,0,3]\n["2","oops","0","3"]\n',
  },
  {
    slug: "list-flatmap",
    output: '["Read","Review","Ship"]\n',
  },
  {
    slug: "list-reverse",
    output: '["Ship","Review","Read"]\n["Read","Review","Ship"]\n',
  },
  {
    slug: "list-tail",
    output: 'missing\npresent: []\npresent: ["Review","Ship"]\n',
  },
  {
    slug: "list-toarray",
    output: '["Read","Review","Ship"]\n["Read","Review","Ship"]\n',
  },
] as const

export const collectionTransformRoutes = [
  {
    identity: "std/array::append",
    module: "std/array",
    namespace: "value",
    kind: "function",
    name: "append",
    slug: "array-append",
    example: "array-append",
    route: "/docs/library/array/function/append/",
  },
  {
    identity: "std/array::concat",
    module: "std/array",
    namespace: "value",
    kind: "function",
    name: "concat",
    slug: "array-concat",
    example: "array-concat",
    route: "/docs/library/array/function/concat/",
  },
  {
    identity: "std/array::filterMap",
    module: "std/array",
    namespace: "value",
    kind: "function",
    name: "filterMap",
    slug: "array-filtermap",
    example: "array-filtermap",
    route: "/docs/library/array/function/filtermap/",
  },
  {
    identity: "std/array::flatMap",
    module: "std/array",
    namespace: "value",
    kind: "function",
    name: "flatMap",
    slug: "array-flatmap",
    example: "array-flatmap",
    route: "/docs/library/array/function/flatmap/",
  },
  {
    identity: "std/array::reverse",
    module: "std/array",
    namespace: "value",
    kind: "function",
    name: "reverse",
    slug: "array-reverse",
    example: "array-reverse",
    route: "/docs/library/array/function/reverse/",
  },
  {
    identity: "std/array::tail",
    module: "std/array",
    namespace: "value",
    kind: "function",
    name: "tail",
    slug: "array-tail",
    example: "array-tail",
    route: "/docs/library/array/function/tail/",
  },
  {
    identity: "std/array::toList",
    module: "std/array",
    namespace: "value",
    kind: "function",
    name: "toList",
    slug: "array-tolist",
    example: "array-tolist",
    route: "/docs/library/array/function/tolist/",
  },
  {
    identity: "std/list::append",
    module: "std/list",
    namespace: "value",
    kind: "function",
    name: "append",
    slug: "list-append",
    example: "list-append",
    route: "/docs/library/list/function/append/",
  },
  {
    identity: "std/list::concat",
    module: "std/list",
    namespace: "value",
    kind: "function",
    name: "concat",
    slug: "list-concat",
    example: "list-concat",
    route: "/docs/library/list/function/concat/",
  },
  {
    identity: "std/list::filterMap",
    module: "std/list",
    namespace: "value",
    kind: "function",
    name: "filterMap",
    slug: "list-filtermap",
    example: "list-filtermap",
    route: "/docs/library/list/function/filtermap/",
  },
  {
    identity: "std/list::flatMap",
    module: "std/list",
    namespace: "value",
    kind: "function",
    name: "flatMap",
    slug: "list-flatmap",
    example: "list-flatmap",
    route: "/docs/library/list/function/flatmap/",
  },
  {
    identity: "std/list::reverse",
    module: "std/list",
    namespace: "value",
    kind: "function",
    name: "reverse",
    slug: "list-reverse",
    example: "list-reverse",
    route: "/docs/library/list/function/reverse/",
  },
  {
    identity: "std/list::tail",
    module: "std/list",
    namespace: "value",
    kind: "function",
    name: "tail",
    slug: "list-tail",
    example: "list-tail",
    route: "/docs/library/list/function/tail/",
  },
  {
    identity: "std/list::toArray",
    module: "std/list",
    namespace: "value",
    kind: "function",
    name: "toArray",
    slug: "list-toarray",
    example: "list-toarray",
    route: "/docs/library/list/function/toarray/",
  },
] as const

export function collectionTransformExamples(playgroundUrl: string) {
  return collectionTransformCases.flatMap(({ slug }) => [
    canonicalExample(
      `collection-transform-${slug}`,
      `apps/site/examples/src/collection-transform/${slug}.ssrg`,
      playgroundUrl
    ),
    canonicalExample(
      `collection-transform-${slug}-ts`,
      `apps/site/examples/src/collection-transform/${slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}

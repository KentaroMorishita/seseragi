import { canonicalExample } from "./canonical-example"

export const mapEditorialCases = [
  {
    name: "empty",
    slug: "empty",
    identity: "std/map::empty",
    expected: "[]\n0\nfalse",
  },
  {
    name: "singleton",
    slug: "singleton",
    identity: "std/map::singleton",
    expected: '[["tea",2]]\n1\n[["empty",""]]',
  },
  {
    name: "fromEntries",
    slug: "fromentries",
    identity: "std/map::fromEntries",
    expected: '[["b",9],["a",1]]\n[]',
  },
  {
    name: "containsKey",
    slug: "containskey",
    identity: "std/map::containsKey",
    expected: "true\nfalse\nfalse",
  },
  {
    name: "insert",
    slug: "insert",
    identity: "std/map::insert",
    expected:
      '[["b",9],["a",1]]\n[["b",9],["a",1],["c",3]]\n[["b",2],["a",1]]\n[["first",1]]',
  },
  {
    name: "upsert",
    slug: "upsert",
    identity: "std/map::upsert",
    expected:
      '[["empty","existing:"],["name","Aki"]]\n[["empty",""],["name","existing:Aki"]]\n[["empty",""],["name","Aki"],["missing","new"]]\n[["empty",""],["name","Aki"]]',
  },
  {
    name: "remove",
    slug: "remove",
    identity: "std/map::remove",
    expected: '[["a",1]]\n[["a",1]]\n[["a",1],["b",8]]\n[["b",2],["a",1]]\n[]',
  },
  {
    name: "keys",
    slug: "keys",
    identity: "std/map::keys",
    expected: '["b","a","c"]\n[]',
  },
  {
    name: "values",
    slug: "values",
    identity: "std/map::values",
    expected: "[2,2,3]\n[]",
  },
  {
    name: "entries",
    slug: "entries",
    identity: "std/map::entries",
    expected: '[["b",2],["a",2],["c",3]]\n[]',
  },
  {
    name: "size",
    slug: "size",
    identity: "std/map::size",
    expected: "2\n2\n3\n0",
  },
  {
    name: "mapValues",
    slug: "mapvalues",
    identity: "std/map::mapValues",
    expected: '[["tea",200],["cake",500]]\n[["tea",2],["cake",5]]\n[]',
  },
  {
    name: "mapKeysWith",
    slug: "mapkeyswith",
    identity: "std/map::mapKeysWith",
    expected:
      '[["same","A/B/C"],["keep","K"]]\n[["first","A"],["keep","K"],["second","B"],["third","C"]]\n[]',
  },
  {
    name: "mergeWith",
    slug: "mergewith",
    identity: "std/map::mergeWith",
    expected:
      '[["b","L-b"],["a","L-a/R-a"],["c","R-c"]]\n[["a","R-a/L-a"],["c","R-c"],["b","L-b"]]\n[["b","L-b"],["a","L-a"]]\n[["a","R-a"],["c","R-c"]]\n[["b","L-b"],["a","L-a"]]\n[["a","R-a"],["c","R-c"]]',
  },
] as const

export function mapEditorialExamples(playgroundUrl: string) {
  return mapEditorialCases.flatMap(({ slug }) => [
    canonicalExample(
      `map-${slug}`,
      `apps/site/examples/src/api-map/${slug}.ssrg`,
      playgroundUrl
    ),
    canonicalExample(
      `map-${slug}-ts`,
      `apps/site/examples/comparisons/api-map/${slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}

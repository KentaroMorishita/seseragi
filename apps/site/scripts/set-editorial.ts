import { canonicalExample } from "./canonical-example"

export const setEditorialCases = [
  {
    name: "empty",
    slug: "empty",
    identity: "std/set::empty",
    expected: "[]\n0\nfalse",
  },
  {
    name: "singleton",
    slug: "singleton",
    identity: "std/set::singleton",
    expected: "[0]\n1\ntrue\n[0]",
  },
  {
    name: "fromIterable",
    slug: "fromiterable",
    identity: "std/set::fromIterable",
    expected: "[3,1,2]\n[3,1,2]\n[]",
  },
  {
    name: "contains",
    slug: "contains",
    identity: "std/set::contains",
    expected: "true\nfalse\nfalse",
  },
  {
    name: "insert",
    slug: "insert",
    identity: "std/set::insert",
    expected: "[3,1,2]\n[3,1,2,4]\n[3,1,2]\n[0]",
  },
  {
    name: "remove",
    slug: "remove",
    identity: "std/set::remove",
    expected: "[3,2]\n[3,2,1]\n[3,1,2]\n[3,1,2]\n[]",
  },
  {
    name: "size",
    slug: "size",
    identity: "std/set::size",
    expected: "3\n3\n4\n0",
  },
  {
    name: "map",
    slug: "map",
    identity: "std/set::map",
    expected: "[1,0]\n[3,2,5,4]\n[]",
  },
  {
    name: "union",
    slug: "union",
    identity: "std/set::union",
    expected: "[3,1,2,4]\n[2,4,1,3]\n[3,1,2]\n[2,4,1]\n[3,1,2]\n[2,4,1]",
  },
  {
    name: "intersection",
    slug: "intersection",
    identity: "std/set::intersection",
    expected: "[1,2]\n[2,1]\n[]\n[]\n[3,1,2]\n[2,4,1]",
  },
  {
    name: "difference",
    slug: "difference",
    identity: "std/set::difference",
    expected: "[3]\n[4]\n[3,1,2]\n[]\n[3,1,2]\n[2,4,1]",
  },
  {
    name: "isSubsetOf",
    slug: "issubsetof",
    identity: "std/set::isSubsetOf",
    expected: "true\nfalse\ntrue\nfalse\ntrue",
  },
] as const

export function setEditorialExamples(playgroundUrl: string) {
  return setEditorialCases.flatMap(({ slug }) => [
    canonicalExample(
      `set-${slug}`,
      `apps/site/examples/src/api-set/${slug}.ssrg`,
      playgroundUrl
    ),
    canonicalExample(
      `set-${slug}-ts`,
      `apps/site/examples/comparisons/api-set/${slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}

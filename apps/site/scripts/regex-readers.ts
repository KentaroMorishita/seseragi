import { canonicalExample } from "./canonical-example"

export const regexReaderCases = [
  {
    slug: "module",
    name: "",
    identity: "std/regex",
    output: "order: ORD-42\nno order code\n",
    typescriptOutput: "order: ORD-42\nno order code\n",
  },
  {
    slug: "compile",
    name: "compile",
    identity: "std/regex::compile",
    output:
      "matches: True\nerror at 3: UnexpectedRegexEnd\nerror at 2: UnsupportedRegexFeature look-around\nmatches: True\n",
    typescriptOutput:
      "matches: true\nrejected pattern\nmatches: false\nmatches: true\n",
  },
  {
    slug: "compilewith",
    name: "compileWith",
    identity: "std/regex::compileWith",
    output: "True\nFalse\nTrue\nTrue\nFalse\n",
    typescriptOutput: "true\nfalse\ntrue\ntrue\nfalse\n",
  },
  {
    slug: "defaultoptions",
    name: "defaultOptions",
    identity: "std/regex::defaultOptions",
    output: "False\nFalse\nFalse\nFalse\n",
    typescriptOutput: "false\nfalse\nfalse\nfalse\n",
  },
  {
    slug: "escape",
    name: "escape",
    identity: "std/regex::escape",
    output: "a\\+b\\.txt\nTrue\nFalse\nTrue\n",
    typescriptOutput: "true\nfalse\ntrue\n",
  },
  {
    slug: "ismatch",
    name: "isMatch",
    identity: "std/regex::isMatch",
    output: "True\nFalse\nTrue\nFalse\nTrue\n",
    typescriptOutput: "true\nfalse\ntrue\nfalse\nfalse\n",
  },
  {
    slug: "find",
    name: "find",
    identity: "std/regex::find",
    output:
      "ORD-42: bytes 3..9; captures=2; tag=absent\nORD-42-X: bytes 3..11; captures=2; tag=X\nno match\n",
    typescriptOutput:
      "ORD-42: UTF-16 2..8; captures=2; tag=absent\nORD-42-X: UTF-16 2..10; captures=2; tag=X\nno match\n",
  },
  {
    slug: "findall",
    name: "findAll",
    identity: "std/regex::findAll",
    output:
      "[0..5:ORD-1, 6..11:ORD-2]\n[]\n[0..0:, 1..1:, 5..5:]\n[1..2:a, 2..3:a]\n[1..3:aa]\n[0..4:👍, 4..8:🏽]\n",
    typescriptOutput:
      "[0..5:ORD-1, 6..11:ORD-2]\n[]\n[0..0:, 1..1:, 3..3:]\n[1..2:a, 2..3:a]\n[1..3:aa]\n[0..2:👍, 2..4:🏽]\n",
  },
  {
    slug: "split",
    name: "split",
    identity: "std/regex::split",
    output: '["a","","b",""]\n["","a",""]\n["","a","👍",""]\n[""]\n',
    typescriptOutput:
      '["a","","b",""]\n["",",","a",",",""]\n["a","👍"]\n[""]\n',
  },
  {
    slug: "replaceall",
    name: "replaceAll",
    identity: "std/regex::replaceAll",
    output: "a$1\\b$1\\\nno digits\n|a|👍|\n|\n",
    typescriptOutput: "a$1\\b$1\\\nno digits\n|a|👍|\n|\na12\\b3\\\n",
  },
  {
    slug: "replaceallwith",
    name: "replaceAllWith",
    identity: "std/regex::replaceAllWith",
    output: "[12] [absent] [3]\nnothing to replace\nempty: []\n",
    typescriptOutput: "[12] [absent] [3]\nnothing to replace\nempty: []\n",
  },
] as const

export function regexReaderExamples(playgroundUrl: string) {
  return regexReaderCases.flatMap(({ slug }) => [
    canonicalExample(
      `regex-reader-${slug}`,
      `apps/site/examples/src/api-regex/${slug}.ssrg`,
      playgroundUrl
    ),
    canonicalExample(
      `regex-reader-${slug}-ts`,
      `apps/site/examples/comparisons/api-regex/${slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}

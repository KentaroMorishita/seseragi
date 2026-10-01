import { canonicalExample } from "./canonical-example"

export const numericReaderCases = [
  {
    slug: "int-module",
    name: "",
    module: "std/int",
    identity: "std/int",
    route: "/docs/library/int/",
    output: "total: 17\ninvalid count\ncount too large\n",
    typescriptOutput: "total: 17\ninvalid count\ncount too large\n",
  },
  {
    slug: "int-parse",
    name: "parse",
    module: "std/int",
    identity: "std/int::parse",
    route: "/docs/library/int/function/parse/",
    output:
      "count: 12\ncount: 0\ninvalid count\ninvalid count\ncount: 9007199254740991\ninvalid count\n",
    typescriptOutput:
      "count: 12\ncount: 0\ninvalid count\ninvalid count\ncount: 9007199254740991\ninvalid count\n",
  },
  {
    slug: "int-format",
    name: "format",
    module: "std/int",
    identity: "std/int::format",
    route: "/docs/library/int/function/format/",
    output: "-12\n0\n12\n",
    typescriptOutput: "-12\n0\n12\n",
  },
  {
    slug: "int-checkedadd",
    name: "checkedAdd",
    module: "std/int",
    identity: "std/int::checkedAdd",
    route: "/docs/library/int/function/checkedadd/",
    output: "result: 17\nout of range\n",
    typescriptOutput: "result: 17\nout of range\n",
  },
  {
    slug: "int-checkedsubtract",
    name: "checkedSubtract",
    module: "std/int",
    identity: "std/int::checkedSubtract",
    route: "/docs/library/int/function/checkedsubtract/",
    output: "result: 7\nout of range\n",
    typescriptOutput: "result: 7\nout of range\n",
  },
  {
    slug: "int-checkedmultiply",
    name: "checkedMultiply",
    module: "std/int",
    identity: "std/int::checkedMultiply",
    route: "/docs/library/int/function/checkedmultiply/",
    output: "result: 60\nout of range\n",
    typescriptOutput: "result: 60\nout of range\n",
  },
  {
    slug: "int-checkeddivide",
    name: "checkedDivide",
    module: "std/int",
    identity: "std/int::checkedDivide",
    route: "/docs/library/int/function/checkeddivide/",
    output: "result: 3\nresult: -3\nresult: -3\nzero divisor\n",
    typescriptOutput: "result: 3\nresult: -3\nresult: -3\nzero divisor\n",
  },
  {
    slug: "int-checkedremainder",
    name: "checkedRemainder",
    module: "std/int",
    identity: "std/int::checkedRemainder",
    route: "/docs/library/int/function/checkedremainder/",
    output: "result: 1\nresult: -1\nresult: 0\nzero divisor\n",
    typescriptOutput: "result: 1\nresult: -1\nresult: 0\nzero divisor\n",
  },
  {
    slug: "float-module",
    name: "",
    module: "std/float",
    identity: "std/float",
    route: "/docs/library/float/",
    output: "per package: 2.5\n",
    typescriptOutput: "per package: 2.5\n",
  },
  {
    slug: "float-parse",
    name: "parse",
    module: "std/float",
    identity: "std/float::parse",
    route: "/docs/library/float/function/parse/",
    output:
      "measurement: 12.5\ninvalid measurement\ninvalid measurement\ninvalid measurement\nnon-finite measurement\nnon-finite measurement\ninvalid measurement\n",
    typescriptOutput:
      "measurement: 12.5\ninvalid measurement\ninvalid measurement\ninvalid measurement\nnon-finite measurement\nnon-finite measurement\ninvalid measurement\n",
  },
  {
    slug: "float-format",
    name: "format",
    module: "std/float",
    identity: "std/float::format",
    route: "/docs/library/float/function/format/",
    output: "12.5\n12.0\n-0.0\n1e21\nNaN\nInfinity\n-Infinity\n",
    typescriptOutput: "12.5\n12\n0\n1e+21\nNaN\nInfinity\n-Infinity\n",
  },
  {
    slug: "float-fromint",
    name: "fromInt",
    module: "std/float",
    identity: "std/float::fromInt",
    route: "/docs/library/float/function/fromint/",
    output: "converted: 5.0\nper package: 2.5\nmaximum: 9007199254740991.0\n",
    typescriptOutput:
      "converted: 5\nper package: 2.5\nmaximum: 9007199254740991\n",
  },
  {
    slug: "float-isfinite",
    name: "isFinite",
    module: "std/float",
    identity: "std/float::isFinite",
    route: "/docs/library/float/function/isfinite/",
    output:
      "finite\nfinite\nfinite\nfinite\nnon-finite\nnon-finite\nnon-finite\n",
    typescriptOutput:
      "finite\nfinite\nfinite\nfinite\nnon-finite\nnon-finite\nnon-finite\n",
  },
] as const

export function numericReaderExamples(playgroundUrl: string) {
  return numericReaderCases.flatMap(({ slug }) => [
    canonicalExample(
      `numeric-reader-${slug}`,
      `apps/site/examples/src/api-numeric-reader/${slug}.ssrg`,
      playgroundUrl
    ),
    canonicalExample(
      `numeric-reader-${slug}-ts`,
      `apps/site/examples/comparisons/api-numeric-reader/${slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}

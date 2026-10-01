import { canonicalExample } from "./canonical-example"

export const dataValidationReaderCases = [
  {
    slug: "maybe-module",
    name: "",
    identity: "std/maybe",
    module: "std/maybe",
    route: "/docs/library/maybe/",
    output: "Present: [Ada]\nNo nickname\nPresent: []\nHello, guest\n",
  },
  {
    slug: "withdefault",
    name: "withDefault",
    identity: "std/maybe::withDefault",
    module: "std/maybe",
    route: "/docs/library/maybe/function/withdefault/",
    output: "Hello, [Ada]\nHello, [guest]\nHello, []\n0\n",
  },
  {
    slug: "orelse",
    name: "orElse",
    identity: "std/maybe::orElse",
    module: "std/maybe",
    route: "/docs/library/maybe/function/orelse/",
    output: "Present: [Ada]\nPresent: [team]\nNo nickname\nPresent: []\n",
  },
  {
    slug: "either-module",
    name: "",
    identity: "std/either",
    module: "std/either",
    route: "/docs/library/either/",
    output: "Seats: 2\nCannot book: seats must be positive\n",
  },
  {
    slug: "fold",
    name: "fold",
    identity: "std/either::fold",
    module: "std/either",
    route: "/docs/library/either/function/fold/",
    output: "Seats: 2\nCannot book: seats must be positive\n",
  },
  {
    slug: "mapleft",
    name: "mapLeft",
    identity: "std/either::mapLeft",
    module: "std/either",
    route: "/docs/library/either/function/mapleft/",
    output: "Cannot book: seats field: seats must be positive\nSeats: 2\n",
  },
  {
    slug: "flatmap",
    name: "flatMap",
    identity: "std/prelude::Monad::flatMap",
    module: "std/prelude",
    route: "/docs/library/prelude/function/flatmap/",
    output:
      "Total: 40\nCannot book: seats must be positive\nCannot book: too many seats\n",
  },
  {
    slug: "validation-module",
    name: "",
    identity: "std/validation",
    module: "std/validation",
    route: "/docs/library/validation/",
    output:
      "Ada: 2\nOK: Ada: 2\nErrors: name is required\nErrors: seats must be positive\nErrors: name is required; seats must be positive\n",
  },
  {
    slug: "valid",
    name: "valid",
    identity: "std/validation::valid",
    module: "std/validation",
    route: "/docs/library/validation/function/valid/",
    output: "OK: Ada\nOK: \n",
  },
  {
    slug: "invalid",
    name: "invalid",
    identity: "std/validation::invalid",
    module: "std/validation",
    route: "/docs/library/validation/function/invalid/",
    output: "Errors: name is required\nOK: Ada\n",
  },
  {
    slug: "invalidmany",
    name: "invalidMany",
    identity: "std/validation::invalidMany",
    module: "std/validation",
    route: "/docs/library/validation/function/invalidmany/",
    output: "Errors: name is required; seats must be positive\n",
  },
  {
    slug: "fromeither",
    name: "fromEither",
    identity: "std/validation::fromEither",
    module: "std/validation",
    route: "/docs/library/validation/function/fromeither/",
    output: "OK: Ada\nErrors: name is required\n",
  },
  {
    slug: "toeither",
    name: "toEither",
    identity: "std/validation::toEither",
    module: "std/validation",
    route: "/docs/library/validation/function/toeither/",
    output: "OK: Ada: 2\nErrors: name is required; seats must be positive\n",
  },
  {
    slug: "maybe-traverse",
    name: "traverse",
    identity: "std/maybe::traverse",
    module: "std/maybe",
    route: "/docs/library/maybe/function/traverse/",
    output: "Seats: [2,1]\nMissing seat\nMissing seat\nSeats: []\n",
  },
  {
    slug: "either-traverse",
    name: "traverse",
    identity: "std/either::traverse",
    module: "std/either",
    route: "/docs/library/either/function/traverse/",
    output:
      "Seats: [2,1]\nError: seats must be positive: 0\nError: seats must be positive: 0\nSeats: []\n",
  },
] as const

export function dataValidationReaderExamples(playgroundUrl: string) {
  return dataValidationReaderCases.flatMap(({ slug }) => [
    canonicalExample(
      `data-validation-reader-${slug}`,
      `apps/site/examples/src/api-data-validation/${slug}.ssrg`,
      playgroundUrl
    ),
    canonicalExample(
      `data-validation-reader-${slug}-ts`,
      `apps/site/examples/comparisons/api-data-validation/${slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}

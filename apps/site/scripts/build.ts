import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"
import { apiCorrectionExamples } from "./api-corrections"
import { arrayEditorialExamples } from "./array-editorial"
import { canonicalExample } from "./canonical-example"
import { charTextExamples } from "./char-text-reader"
import { collectionTypeExamples } from "./collection-type-readers"
import { comparisonExamples } from "./comparisons"
import { referenceCoverage } from "./coverage"
import { dataChoiceExamples } from "./data-choices"
import { dataOperationExamples } from "./data-operations"
import { dataValidationReaderExamples } from "./data-validation-readers"
import { failureReaderExamples } from "./failure-readers"
import { filesystemReaderExamples } from "./filesystem-reader"
import { lifecycleReaderExamples } from "./lifecycle-readers"
import { listEditorialExamples } from "./list-editorial"
import { mapEditorialExamples } from "./map-editorial"
import { modelConceptExamples } from "./model-concepts"
import { newModelReaderExamples } from "./model-reader"
import { moduleProjectExamples } from "./module-examples"
import { nonemptyIteratorReaderExamples } from "./nonempty-iterator-readers"
import { numericReaderExamples } from "./numeric-readers"
import { compilerReferenceModules } from "./reference"
import { regexReaderExamples } from "./regex-readers"
import { type RenderedPage, renderGenerator } from "./render-generator"
import { resultFoundationExamples } from "./result-foundation-readers"
import { sequenceEditorialExamples } from "./sequence-editorial"
import { setEditorialExamples } from "./set-editorial"
import { syntaxExamples } from "./syntax-examples"
import { textEditorialExamples } from "./text-editorial"
import { traitReaderExamples } from "./trait-readers"
import { typeLimitExamples } from "./type-limits"
import { typeReaderExamples } from "./type-readers"
import { unicodeReaderExamples } from "./unicode-reader"
import { webReaderExamples } from "./web-reader"

const app = resolve(import.meta.dir, "..")
const root = resolve(app, "../..")
const styleFiles = [
  "tokens.css",
  "base.css",
  "shell.css",
  "language-menu.css",
  "home.css",
  "docs.css",
  "mobile-navigation.css",
  "article.css",
  "code.css",
  "responsive.css",
]

type BuildOptions = {
  output: string
  origin: string
  playgroundUrl?: string
  profile?: "development" | "release"
}

const sha256 = (value: string | Buffer) =>
  createHash("sha256").update(value).digest("hex")

export function generatorInput(playgroundUrl: string) {
  return {
    schema: 1,
    origin: "",
    playgroundUrl,
    tourUrl: new URL("tour/", playgroundUrl).href,
    grammar: readFileSync(resolve(root, "docs/spec/grammar.md"), "utf8")
      .split("```ebnf\n")[1]
      .split("```")[0]
      .trimEnd(),
    examples: [
      ...apiCorrectionExamples(playgroundUrl),
      ...arrayEditorialExamples(playgroundUrl),
      ...comparisonExamples(playgroundUrl),
      ...dataChoiceExamples(playgroundUrl),
      ...dataOperationExamples(playgroundUrl),
      ...failureReaderExamples(playgroundUrl),
      ...lifecycleReaderExamples(playgroundUrl),
      ...listEditorialExamples(playgroundUrl),
      ...mapEditorialExamples(playgroundUrl),
      ...unicodeReaderExamples(playgroundUrl),
      ...regexReaderExamples(playgroundUrl),
      ...setEditorialExamples(playgroundUrl),
      ...nonemptyIteratorReaderExamples(playgroundUrl),
      ...charTextExamples(playgroundUrl),
      ...collectionTypeExamples(playgroundUrl),
      ...dataValidationReaderExamples(playgroundUrl),
      ...numericReaderExamples(playgroundUrl),
      ...webReaderExamples(playgroundUrl),
      ...filesystemReaderExamples(playgroundUrl),
      ...resultFoundationExamples(playgroundUrl),
      ...moduleProjectExamples(playgroundUrl),
      ...newModelReaderExamples(playgroundUrl),
      ...modelConceptExamples(playgroundUrl),
      ...textEditorialExamples(playgroundUrl),
      ...syntaxExamples(playgroundUrl),
      ...sequenceEditorialExamples(playgroundUrl),
      ...typeReaderExamples(playgroundUrl),
      ...typeLimitExamples(playgroundUrl),
      ...traitReaderExamples(playgroundUrl),
      canonicalExample(
        "pilot-built-in-types",
        "apps/site/examples/src/language/pilot-built-in-types.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "pilot-annotations",
        "apps/site/examples/src/language/pilot-annotations.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "pilot-function-application",
        "apps/site/examples/src/language/pilot-function-application.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "pilot-blocks",
        "apps/site/examples/src/language/pilot-blocks.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "pilot-currying",
        "apps/site/examples/src/language/pilot-currying.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "pilot-immutable-invalid",
        "apps/site/examples/invalid/src/language/pilot-immutable.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "pilot-built-in-types-invalid",
        "apps/site/examples/invalid/src/language/pilot-built-in-types.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "pilot-annotations-invalid",
        "apps/site/examples/invalid/src/language/pilot-annotations.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "pilot-function-application-invalid",
        "apps/site/examples/invalid/src/language/pilot-function-application.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "pilot-blocks-invalid",
        "apps/site/examples/invalid/src/language/pilot-blocks.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "pilot-currying-invalid",
        "apps/site/examples/invalid/src/language/pilot-currying.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "principle-expression-oriented",
        "apps/site/examples/src/language/principle-expression-oriented.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "principle-immutable-by-default",
        "apps/site/examples/src/language/principle-immutable-by-default.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "principle-no-hidden-danger",
        "apps/site/examples/src/language/principle-no-hidden-danger.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "principle-backend-independent-semantics",
        "apps/site/examples/src/language/principle-backend-independent-semantics.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "principle-diagnosable-behavior",
        "apps/site/examples/src/language/principle-diagnosable-behavior.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "principle-visible-costs",
        "apps/site/examples/src/language/principle-visible-costs.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "principle-readable-density",
        "apps/site/examples/src/language/principle-readable-density.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-variance-invalid",
        "apps/site/examples/invalid/src/language/types-variance.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-kind-invalid",
        "apps/site/examples/invalid/src/language/invalid-types-kind.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-generic-function-invalid",
        "apps/site/examples/invalid/src/language/invalid-types-generic-function.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-generic-adt-invalid",
        "apps/site/examples/invalid/src/language/invalid-types-generic-adt.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-generic-struct-invalid",
        "apps/site/examples/invalid/src/language/invalid-types-generic-struct.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-generic-alias-invalid",
        "apps/site/examples/invalid/src/language/invalid-types-generic-alias.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-newtype-invalid",
        "apps/site/examples/invalid/src/language/invalid-types-newtype.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "modules-re-exports",
        "apps/site/examples/src/language/modules-re-exports.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "modules-namespaces",
        "apps/site/examples/src/language/modules-namespaces.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "modules-initialization",
        "apps/site/examples/src/language/modules-initialization.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-kinds",
        "apps/site/examples/src/language/types-kinds.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-scope",
        "apps/site/examples/src/language/types-scope.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-generic-functions",
        "apps/site/examples/src/language/types-generic-functions.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-let-rank",
        "apps/site/examples/src/language/types-let-rank.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-generic-adts",
        "apps/site/examples/src/language/types-generic-adts.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-generic-structs",
        "apps/site/examples/src/language/types-generic-structs.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-generic-methods",
        "apps/site/examples/src/language/types-generic-methods.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-generic-aliases",
        "apps/site/examples/src/language/types-generic-aliases.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-newtypes",
        "apps/site/examples/src/language/types-newtypes.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-variance",
        "apps/site/examples/src/language/types-variance.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-erasure",
        "apps/site/examples/src/language/types-erasure.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "modules-domain",
        "apps/site/examples/src/language/modules-domain.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "modules-identity",
        "apps/site/examples/src/language/modules-identity.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "modules-imports",
        "apps/site/examples/src/language/modules-imports.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "modules-top-level",
        "apps/site/examples/src/language/modules-top-level.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "modules-private-domain",
        "apps/site/examples/invalid/visibility/src/domain.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "modules-invalid-private-import",
        "apps/site/examples/invalid/visibility/src/main.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "modules-invalid-import",
        "apps/site/examples/invalid/import/src/main.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "hello-world",
        "examples/samples/hello-world/main.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "function-application",
        "apps/site/examples/src/language/function-application.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-method-calls",
        "apps/site/examples/src/language/method-calls.ssrg",
        playgroundUrl
      ),
      ...[
        "method-calls",
        "pipelines",
        "records",
        "structs",
        "collections",
        "trait",
      ].map((name) =>
        canonicalExample(
          `reader-${name}`,
          `apps/site/examples/src/language/reader-${name}.ssrg`,
          playgroundUrl
        )
      ),
      canonicalExample(
        "syntax-invalid-methods",
        "apps/site/examples/invalid/src/language/invalid-methods.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-pipelines",
        "apps/site/examples/src/language/pipelines.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-operator-precedence",
        "apps/site/examples/src/language/operator-precedence.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-custom-operator",
        "apps/site/examples/src/language/custom-operator.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-invalid-custom-operator",
        "apps/site/examples/invalid/src/language/invalid-custom-operator.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-reserved-words",
        "apps/site/examples/src/language/reserved-words.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-optional-record-field",
        "apps/site/examples/src/language/optional-record-field.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "syntax-invalid-record-fields",
        "apps/site/examples/invalid/src/language/invalid-record-fields.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-system",
        "apps/site/examples/src/language/type-system.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-constructors",
        "apps/site/examples/src/language/type-constructors.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-polymorphism",
        "apps/site/examples/src/language/polymorphism.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-nominal-structural",
        "apps/site/examples/src/language/nominal-and-structural.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-invalid-structural",
        "apps/site/examples/invalid/src/language/invalid-structural.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-requirement-merge",
        "apps/site/examples/src/language/requirement-merge.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-invalid-requirement-merge",
        "apps/site/examples/invalid/src/language/invalid-requirement-merge.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-invalid-coercion",
        "apps/site/examples/invalid/src/language/invalid-coercion.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-recursion",
        "apps/site/examples/src/language/recursion.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "types-invalid-recursion",
        "apps/site/examples/invalid/src/language/invalid-recursion.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "language-program-entry",
        "apps/site/examples/src/language/program-entry.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-source-text",
        "apps/site/examples/src/language/source-text.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-literals",
        "apps/site/examples/src/language/literals.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-invalid-numeric",
        "apps/site/examples/invalid/src/language/invalid-numeric.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-char-valid",
        "apps/site/examples/src/language/character-literals.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-char-invalid",
        "apps/site/examples/invalid/src/language/invalid-character-literals.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-invalid-escape",
        "apps/site/examples/invalid/src/language/invalid-escape.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "expressions-evaluation",
        "apps/site/examples/src/language/expressions-evaluation.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "expressions-blocks",
        "apps/site/examples/src/language/expressions-blocks.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "expressions-conditionals",
        "apps/site/examples/src/language/expressions-conditionals.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "expressions-invalid-conditional",
        "apps/site/examples/invalid/src/language/invalid-conditional.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "expressions-ranges-comprehensions",
        "apps/site/examples/src/language/expressions-ranges-comprehensions.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "expressions-lambdas",
        "apps/site/examples/src/language/expressions-lambdas.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-algebraic-data-types",
        "apps/site/examples/src/language/data-algebraic-data-types.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "data-structs",
        "apps/site/examples/src/language/data-structs.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-invalid-struct",
        "apps/site/examples/invalid/src/language/invalid-struct-literal.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-newtypes",
        "apps/site/examples/src/language/data-newtypes.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-records",
        "apps/site/examples/src/language/data-records.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-invalid-record",
        "apps/site/examples/invalid/src/language/invalid-record-literal.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-tuples-arrays-lists",
        "apps/site/examples/src/language/data-tuples-arrays-lists.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-impls-and-methods",
        "apps/site/examples/src/language/data-impls-and-methods.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-invalid-impl-owner",
        "apps/site/examples/invalid/src/language/invalid-impl-owner.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-operator-overloads",
        "apps/site/examples/src/language/data-operator-overloads.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-invalid-operator-overload",
        "apps/site/examples/invalid/src/language/invalid-operator-overload.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "patterns-binding-rules",
        "apps/site/examples/src/language/patterns-binding-rules.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "patterns-invalid-binding",
        "apps/site/examples/invalid/src/language/invalid-binding-pattern.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "patterns-irrefutable",
        "apps/site/examples/src/language/patterns-irrefutable.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "patterns-invalid-irrefutable",
        "apps/site/examples/invalid/src/language/invalid-irrefutable-pattern.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "patterns-match",
        "apps/site/examples/src/language/patterns-match.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "patterns-invalid-match",
        "apps/site/examples/invalid/src/language/invalid-match.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-model",
        "apps/site/examples/src/language/traits-model.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "traits-declarations",
        "apps/site/examples/src/language/traits-declarations.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "traits-invalid-declaration",
        "apps/site/examples/invalid/src/language/invalid-trait-declaration.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-instances",
        "apps/site/examples/src/language/traits-instances.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "traits-invalid-instance",
        "apps/site/examples/invalid/src/language/invalid-trait-instance.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-constraints",
        "apps/site/examples/src/language/traits-constraints.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "traits-invalid-constraint",
        "apps/site/examples/invalid/src/language/invalid-trait-constraint.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-method-calls",
        "apps/site/examples/src/language/traits-method-calls.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "traits-invalid-method",
        "apps/site/examples/invalid/src/language/invalid-trait-method.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-coherence",
        "apps/site/examples/src/language/traits-coherence.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "traits-invalid-coherence",
        "apps/site/examples/invalid/src/language/invalid-trait-coherence.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-standard-operators",
        "apps/site/examples/src/language/traits-standard-operators.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "traits-laws",
        "apps/site/examples/src/language/traits-laws.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "traits-deriving",
        "apps/site/examples/src/language/traits-deriving.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "traits-invalid-deriving",
        "apps/site/examples/invalid/src/language/invalid-trait-deriving.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-methods-versus-traits",
        "apps/site/examples/src/language/traits-methods-versus-traits.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "traits-do-notation",
        "apps/site/examples/src/language/traits-do-notation.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "traits-do-desugaring",
        "apps/site/examples/src/language/traits-do-desugaring.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "traits-do-block-typing",
        "apps/site/examples/src/language/traits-do-block-typing.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "traits-invalid-do",
        "apps/site/examples/invalid/src/language/invalid-trait-do.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-pure-expressions",
        "apps/site/examples/src/language/effects-pure-expressions.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-maybe",
        "apps/site/examples/src/language/effects-maybe.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-either",
        "apps/site/examples/src/language/effects-either.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-cold-value",
        "apps/site/examples/src/language/effects-cold-value.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-contract-form",
        "apps/site/examples/src/language/effects-contract-form.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-inferred-form",
        "apps/site/examples/src/language/effects-inferred-form.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-effectful-for",
        "apps/site/examples/src/language/effects-effectful-for.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-environment",
        "apps/site/examples/src/language/effects-environment.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-error-channels",
        "apps/site/examples/src/language/effects-error-channels.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-task",
        "apps/site/examples/src/language/effects-task.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-execution-order",
        "apps/site/examples/src/language/effects-execution-order.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-runtime-boundary",
        "apps/site/examples/src/language/effects-runtime-boundary.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-defects",
        "apps/site/examples/src/language/effects-defects.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-invalid-pure-body",
        "apps/site/examples/invalid/src/language/invalid-effect-pure-body.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-maybe",
        "apps/site/examples/invalid/src/language/invalid-effect-maybe.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-contract",
        "apps/site/examples/invalid/src/language/invalid-effect-contract.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-errors",
        "apps/site/examples/invalid/src/language/invalid-effect-errors.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-task",
        "apps/site/examples/invalid/src/language/invalid-effect-task.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-cancellation-resources",
        "apps/site/examples/src/language/effects-cancellation-resources.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-scheduler-fairness",
        "apps/site/examples/src/language/effects-scheduler-fairness.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-fiber-supervision",
        "apps/site/examples/src/language/effects-fiber-supervision.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-signal-transactions",
        "apps/site/examples/src/language/effects-signal-transactions.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-derived-signals",
        "apps/site/examples/src/language/effects-derived-signals.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-signal-operators",
        "apps/site/examples/src/language/effects-signal-operators.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-signal-subscription",
        "apps/site/examples/src/language/effects-signal-subscription.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-foreign-failure",
        "apps/site/examples/src/language/effects-foreign-failure.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "effects-invalid-finalizer",
        "apps/site/examples/invalid/src/language/invalid-effect-finalizer.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-signal-read",
        "apps/site/examples/invalid/src/language/invalid-signal-read.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-signal-distinct",
        "apps/site/examples/invalid/src/language/invalid-signal-distinct.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-signal-write",
        "apps/site/examples/invalid/src/language/invalid-signal-write.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-throw",
        "apps/site/examples/invalid/src/language/invalid-effect-throw.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-layout",
        "apps/site/examples/src/language/syntax-layout.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "web-starter-main",
        "examples/samples/web-starter/src/main.ssrg",
        playgroundUrl
      ),
    ],
    referenceModules: compilerReferenceModules(),
  }
}

function routeFile(output: string, route: string): string {
  assert.match(route, /^\/(?:[a-z0-9-]+\/)*$/u)
  return join(output, route.slice(1), "index.html")
}

export function validateInternalLinks(pages: RenderedPage[]) {
  const byRoute = new Map<string, { ids: Set<string>; links: string[] }>()
  for (const page of pages) {
    const ids: string[] = []
    const links: string[] = []
    // Read elements, not escaped example output, comments or script text.
    new HTMLRewriter()
      .on("[id]", {
        element(element) {
          const id = element.getAttribute("id")
          if (id !== null) ids.push(id)
        },
      })
      .on("a[href]", {
        element(element) {
          const href = element.getAttribute("href")
          if (href !== null) links.push(href)
        },
      })
      .transform(page.html)
    assert.equal(
      new Set(ids).size,
      ids.length,
      `Duplicate HTML id in ${page.route}`
    )
    byRoute.set(page.route, { ids: new Set(ids), links })
  }
  for (const page of pages) {
    for (const href of byRoute.get(page.route)?.links ?? []) {
      if (href.startsWith("http://") || href.startsWith("https://")) continue
      const [path, fragment] = href.split("#", 2)
      const route = path || page.route
      const target = byRoute.get(route)
      assert.ok(target, `Unresolved internal link from ${page.route}: ${href}`)
      if (fragment)
        assert.ok(
          target.ids.has(fragment),
          `Unresolved fragment from ${page.route}: ${href}`
        )
    }
  }
}

export function compileGenerator(
  directory: string,
  profile: "development" | "release"
): string {
  const cli =
    process.env.SESERAGI_BIN ?? join(root, "target", "debug", "seseragi")
  const result = spawnSync(
    cli,
    [
      "build",
      app,
      "--target",
      "process",
      "--profile",
      profile,
      "--out-dir",
      directory,
    ],
    { cwd: root, encoding: "utf8" }
  )
  assert.equal(result.status, 0, result.stderr || result.stdout)
  const entry = join(directory, profile === "release" ? "entry.js" : "entry.ts")
  assert.ok(existsSync(entry), `Missing ${profile} generator entry: ${entry}`)
  return entry
}

function publishAssets(output: string): string[] {
  const assets = join(output, "assets")
  mkdirSync(assets, { recursive: true })
  const css = [
    readFileSync(
      join(root, "apps/playground/src/editor/syntax-theme.css"),
      "utf8"
    ).trim(),
    ...styleFiles.map((name) =>
      readFileSync(join(app, "styles", name), "utf8").trim()
    ),
  ].join("\n\n")
  writeFileSync(join(assets, "site.css"), `${css}\n`)
  copyFileSync(
    join(root, "assets/brand/public/brand/seseragi-icon.svg"),
    join(assets, "seseragi-icon.svg")
  )
  const client = new Bun.Transpiler({ loader: "ts" }).transformSync(
    readFileSync(join(app, "client/mobile-navigation.ts"), "utf8")
  )
  writeFileSync(join(assets, "mobile-navigation.js"), client)
  copyFileSync(join(app, "public/language.svg"), join(assets, "language.svg"))
  writeFileSync(
    join(assets, "language-menu.js"),
    new Bun.Transpiler({ loader: "ts" }).transformSync(
      readFileSync(join(app, "client/language-menu.ts"), "utf8")
    )
  )
  return [
    "assets/seseragi-icon.svg",
    "assets/site.css",
    "assets/mobile-navigation.js",
    "assets/language.svg",
    "assets/language-menu.js",
  ]
}

export function buildSite(options: BuildOptions) {
  const output = resolve(options.output)
  assert.ok(!existsSync(output), `Output already exists: ${output}`)
  const origin = new URL(options.origin)
  assert.equal(origin.pathname, "/", "Site origin must not contain a path")
  assert.equal(origin.protocol, "https:", "Site origin must use HTTPS")
  const playgroundUrl = options.playgroundUrl ?? "https://seseragi.vercel.app/"
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-site-"))
  try {
    const generator = join(temporary, "generator")
    const entry = compileGenerator(generator, options.profile ?? "development")
    const input = { ...generatorInput(playgroundUrl), origin: origin.origin }
    const pages = renderGenerator(entry, input)
    const referencePageCount = input.referenceModules.reduce(
      (count, module) => count + 1 + module.items.length,
      0
    )
    assert.equal(
      pages.length,
      2 * (113 + referencePageCount),
      "Unexpected bilingual page count"
    )
    assert.equal(
      new Set(pages.map(({ route }) => route)).size,
      pages.length,
      "Duplicate generated route"
    )
    validateInternalLinks(pages)
    const coverage = referenceCoverage(pages.map(({ route }) => route))
    for (const { route, html } of pages) {
      if (!route.startsWith("/ja/")) continue
      const main = html.match(/<main\b[^>]*>([\s\S]*?)<\/main>/u)?.[1]
      assert.ok(main, `Missing article main: ${route}`)
      assert.ok(
        !/準備中|整備中|詳しい日本語解説|日本語本文は#[0-9]+/u.test(main),
        `Untranslated article placeholder: ${route}`
      )
    }
    mkdirSync(output, { recursive: true })
    for (const page of pages) {
      assert.ok(!page.html.includes("site-build-error"), page.route)
      const path = routeFile(output, page.route)
      mkdirSync(dirname(path), { recursive: true })
      // Browser-only enhancement is linked by the host publisher, not embedded
      // in page prose. Content, navigation and component markup stay in Seseragi.
      const html = page.html.includes('class="mobile-docs-navigation"')
        ? page.html.replace(
            "</body>",
            '<script type="module" src="/assets/mobile-navigation.js"></script></body>'
          )
        : page.html
      writeFileSync(
        path,
        html.replace(
          "</body>",
          '<script type="module" src="/assets/language-menu.js"></script></body>'
        )
      )
    }
    const assets = publishAssets(output)
    const files = [
      ...pages.map(({ route }) =>
        route === "/" ? "index.html" : `${route.slice(1)}index.html`
      ),
      ...assets,
    ].sort()
    const manifest = {
      schema: 1,
      generator: "seseragi/official-site",
      referenceCoverage: coverage,
      pages: pages.map(({ route }) => route).sort(),
      examples: input.examples.map(({ id, sourcePath, sha256 }) => ({
        id,
        sourcePath,
        sha256,
      })),
      referenceModules: input.referenceModules.map(
        ({ specifier, availability, targets, items }) => ({
          specifier,
          availability,
          targets,
          symbols: items.map(({ identity, namespace, itemKind }) => ({
            identity,
            namespace,
            itemKind,
          })),
        })
      ),
      files: files.map((path) => ({
        path,
        sha256: sha256(readFileSync(join(output, path))),
      })),
    }
    writeFileSync(
      join(output, "site-manifest.json"),
      `${JSON.stringify(manifest, null, 2)}\n`
    )
    return manifest
  } catch (error) {
    rmSync(output, { recursive: true, force: true })
    throw error
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
}

if (import.meta.main) {
  const [output, originArgument] = process.argv.slice(2)
  const origin =
    originArgument ??
    process.env.SESERAGI_SITE_ORIGIN ??
    "https://seseragi-docs.vercel.app"
  assert.ok(output, "Usage: build.ts OUTPUT [ORIGIN]")
  const finalOutput = resolve(output)
  const stagingOutput = `${finalOutput}.staging-${process.pid}`
  try {
    const manifest = buildSite({
      output: stagingOutput,
      origin,
      profile:
        process.env.NODE_ENV === "production" ? "release" : "development",
    })
    rmSync(finalOutput, { recursive: true, force: true })
    renameSync(stagingOutput, finalOutput)
    console.log(
      JSON.stringify(
        { output: finalOutput, pages: manifest.pages.length },
        null,
        2
      )
    )
  } catch (error) {
    rmSync(stagingOutput, { recursive: true, force: true })
    throw error
  }
}

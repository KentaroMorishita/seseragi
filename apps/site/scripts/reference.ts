import assert from "node:assert/strict"
import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { highlightSeseragi } from "../../playground/src/editor/seseragi-language"
import { bytesInspectionReading } from "./bytes-inspection-reading"
import { bytesReaderReading } from "./bytes-reader-reading"
import { effectSequencingReading } from "./effect-sequencing-reading"
import { filesystemReaderReading } from "./filesystem-reader-reading"
import { jsonReaderReading } from "./json-reader-reading"
import { referenceDescription } from "./reference-copy"
import { resultFoundationReading } from "./result-foundation-reading"
import { signatureReading } from "./signature-reading"
import { stdinReaderReading } from "./stdin-reader-reading"
import { terminalReaderReading } from "./terminal-reader-reading"

const root = resolve(import.meta.dir, "../../..")
const referencePath = join(
  root,
  "examples/spec/artifacts/stdlib-schema-1/reference/module.json"
)
const preludePath = join(
  root,
  "examples/spec/artifacts/stdlib-schema-1/prelude/module.json"
)

type CompilerReferenceItem = {
  identity: string
  name: string
  module: string
  category: string
  kind: string
  signature: string
  description: string
  typeParameters: string[]
  constraints: string[]
  namespace: string
  instanceClass?: string
  instanceHead?: string
}

type CompilerReferenceModule = {
  specifier: string
  availability: string
  targets: string[]
  items: CompilerReferenceItem[]
}

type CompilerTypeParameter = string | { name: string; arity: number }

type CompilerTrait = {
  name: string
  typeParameters: CompilerTypeParameter[]
}

type CompilerInstanceConstraint = {
  trait: string
  typeArgumentIndex?: number
  typeArgumentIndices?: number[]
}

type CompilerInstance = {
  trait: string
  typeConstructor: string
  typeConstructorCanonical: string
  typeConstructorArity: number
  identity: string
  constraints?: CompilerInstanceConstraint[]
}

type CompilerBuiltinInstance = {
  trait: string
  arguments: string[]
  identity: string
  dispatch: "dictionary" | "operator-abi"
}

type CompilerInstanceAuditRow = {
  trait: string
  head: string
  identity?: string
  status: string
  classification: string
}

type CompilerPrelude = {
  schema: number
  kind: string
  traits: CompilerTrait[]
  instances: CompilerInstance[]
  builtinInstances: CompilerBuiltinInstance[]
  instanceAudit: { matrix: CompilerInstanceAuditRow[] }
}

function typeParameterNames(count: number): string[] {
  const names = ["A", "B", "C", "D", "E", "F", "G", "H"]
  return Array.from(
    { length: count },
    (_, index) => names[index] ?? `T${index + 1}`
  )
}

function typeApplication(name: string, arguments_: string[]): string {
  return arguments_.length === 0 ? name : `${name}<${arguments_.join(", ")}>`
}

function instanceSignature(
  name: string,
  typeParameters: string[],
  constraints: string[]
): string {
  const parameters =
    typeParameters.length === 0 ? "" : `<${typeParameters.join(", ")}>`
  const requirements =
    constraints.length === 0 ? "" : ` where ${constraints.join(", ")}`
  return `instance${parameters} ${name}${requirements}`
}

function moduleFromIdentity(identity: string): string | undefined {
  return identity.startsWith("std/") ? identity.split("::", 1)[0] : undefined
}

function moduleFromCanonical(canonical: string): string {
  return canonical.split("::", 1)[0] ?? "std/prelude"
}

function variablesInArguments(arguments_: string[]): string[] {
  const variables: string[] = []
  for (const argument of arguments_) {
    for (const token of argument.split(/[^A-Za-z0-9]+/u)) {
      if (/^[A-Z]$/u.test(token) && !variables.includes(token))
        variables.push(token)
    }
  }
  return variables
}

function instanceItem(
  identity: string,
  preferredModule: string,
  name: string,
  signature: string,
  instanceClass: string,
  instanceHead: string,
  typeParameters: string[],
  constraints: string[]
): CompilerReferenceItem & { preferredModule: string } {
  return {
    identity,
    preferredModule,
    name,
    module: preferredModule,
    category: "Instances",
    kind: "instance",
    signature,
    description: "",
    typeParameters,
    constraints,
    namespace: "instance",
    instanceClass,
    instanceHead,
  }
}

function compilerReferenceInstances(): Array<
  CompilerReferenceItem & { preferredModule: string }
> {
  const artifact = JSON.parse(
    readFileSync(preludePath, "utf8")
  ) as CompilerPrelude
  assert.equal(artifact.schema, 1)
  assert.equal(artifact.kind, "standard-module-surface")
  const traits = new Map(artifact.traits.map((trait_) => [trait_.name, trait_]))
  const instances = artifact.instances.map((instance) => {
    const trait_ = traits.get(instance.trait)
    assert.ok(trait_, `Unknown instance trait: ${instance.trait}`)
    assert.equal(
      trait_.typeParameters.length,
      1,
      `Registered instance trait must be unary: ${instance.trait}`
    )
    const parameter = trait_.typeParameters[0]
    const remainingArity = typeof parameter === "string" ? 0 : parameter.arity
    const supplied = instance.typeConstructorArity - remainingArity
    assert.ok(supplied >= 0, `Invalid instance head: ${instance.identity}`)
    const typeParameters = typeParameterNames(supplied)
    const head = typeApplication(instance.typeConstructor, typeParameters)
    const name = `${instance.trait}<${head}>`
    const constraints = (instance.constraints ?? []).map((constraint) => {
      const indices =
        constraint.typeArgumentIndices ??
        (constraint.typeArgumentIndex === undefined
          ? []
          : [constraint.typeArgumentIndex])
      assert.ok(indices.length > 0, `Empty constraint: ${instance.identity}`)
      const arguments_ = indices.map((index) => {
        const value = typeParameters[index]
        assert.ok(value, `Constraint index outside ${instance.identity}`)
        return value
      })
      return typeApplication(constraint.trait, arguments_)
    })
    return instanceItem(
      instance.identity,
      moduleFromIdentity(instance.identity) ??
        moduleFromCanonical(instance.typeConstructorCanonical),
      name,
      instanceSignature(name, typeParameters, constraints),
      constraints.length === 0 ? "standard" : "conditional",
      head,
      typeParameters,
      constraints
    )
  })

  instances.push(
    ...artifact.builtinInstances.map((instance) => {
      const typeParameters = variablesInArguments(instance.arguments)
      const name = `${instance.trait}<${instance.arguments.join(", ")}>`
      return instanceItem(
        instance.identity,
        moduleFromIdentity(instance.identity) ?? "std/prelude",
        name,
        instanceSignature(name, typeParameters, []),
        instance.dispatch,
        instance.arguments.join(", "),
        typeParameters,
        []
      )
    }),
    ...artifact.instanceAudit.matrix
      .filter(
        (row) =>
          row.status === "specified-and-implemented" &&
          row.classification === "structural"
      )
      .map((row) => {
        assert.ok(
          row.identity,
          `Structural instance has no identity: ${row.trait}`
        )
        const name = `${row.trait}<${row.head}>`
        return instanceItem(
          row.identity,
          "std/prelude",
          name,
          instanceSignature(name, [], []),
          "structural",
          row.head,
          [],
          []
        )
      })
  )
  assert.equal(instances.length, 357, "Unexpected compiler instance count")
  assert.equal(
    new Set(instances.map(({ identity }) => identity)).size,
    instances.length,
    "Duplicate compiler instance identity"
  )
  return instances
}

export function compilerReferenceModules() {
  const artifact = JSON.parse(readFileSync(referencePath, "utf8"))
  assert.equal(artifact.schema, 1)
  assert.ok(
    Array.isArray(artifact.modules),
    "Reference modules must be an array"
  )
  const modules = artifact.modules as CompilerReferenceModule[]
  assert.equal(
    new Set(modules.map(({ specifier }) => specifier)).size,
    modules.length,
    "Duplicate compiler reference module"
  )
  const moduleIndexes = new Map(
    modules.map((module, index) => [module.specifier, index])
  )
  const preludeInstanceNamespaces = new Set([
    "std/bool",
    "std/product",
    "std/range",
    "std/string",
    "std/sum",
    "std/unit",
  ])
  for (const instance of compilerReferenceInstances()) {
    const target = moduleIndexes.get(instance.preferredModule)
    const moduleIndex =
      target ??
      (preludeInstanceNamespaces.has(instance.preferredModule)
        ? moduleIndexes.get("std/prelude")
        : undefined)
    assert.notEqual(
      moduleIndex,
      undefined,
      `Unknown instance module: ${instance.preferredModule}`
    )
    const module = modules[moduleIndex]
    module.items.push({ ...instance, module: module.specifier })
  }
  for (const module of modules)
    module.items.sort((left, right) =>
      [left.category, left.name, left.identity, left.kind]
        .join("\u0000")
        .localeCompare(
          [right.category, right.name, right.identity, right.kind].join(
            "\u0000"
          )
        )
    )
  const symbolKeys = modules.flatMap(({ items }) =>
    items.map(
      ({ identity, namespace, kind }) =>
        `${identity}\u0000${namespace}\u0000${kind}`
    )
  )
  assert.equal(
    new Set(symbolKeys).size,
    symbolKeys.length,
    "Duplicate compiler reference symbol key"
  )
  return modules.map((module) => {
    assert.match(module.specifier, /^std\/[-a-z0-9/]+$/u)
    assert.ok(
      module.items.length > 0,
      `Empty reference module: ${module.specifier}`
    )
    return {
      specifier: module.specifier,
      availability: module.availability,
      targets: module.targets,
      items: module.items.map((item) => {
        assert.equal(item.module, module.specifier, item.identity)
        const description =
          item.description === ""
            ? { en: "", ja: "" }
            : referenceDescription(item.description)
        return {
          identity: item.identity,
          name: item.name,
          module: item.module,
          category: item.category,
          namespace: item.namespace,
          itemKind: item.kind,
          signature: item.signature,
          description: item.description,
          descriptionEn: description.en,
          descriptionJa: description.ja,
          reading: bytesInspectionReading(
            item,
            stdinReaderReading(
              item,
              terminalReaderReading(
                item,
                bytesReaderReading(
                  item,
                  jsonReaderReading(
                    item,
                    effectSequencingReading(
                      item,
                      filesystemReaderReading(
                        item,
                        resultFoundationReading(
                          item,
                          signatureReading(
                            item.signature,
                            item.kind,
                            item.typeParameters,
                            item.constraints
                          )
                        )
                      )
                    )
                  )
                )
              )
            )
          ),
          typeParameters: item.typeParameters,
          constraints: item.constraints,
          instanceClass: item.instanceClass ?? "",
          instanceHead: item.instanceHead ?? "",
          highlighted: highlightSeseragi(item.signature).map(
            ({ text, classes }) => ({ text, className: classes })
          ),
        }
      }),
    }
  })
}

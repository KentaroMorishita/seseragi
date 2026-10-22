import {
  type BrowserDom,
  createBrowserDom,
} from "../../../../runtime/ts/src/browser/dom"
import { createBrowserEnvironment } from "../../../../runtime/ts/src/browser/host"
import {
  createBrowserNavigationProvider,
  createWindowNavigationHost,
} from "../../../../runtime/ts/src/browser/provider-navigation"
import {
  createBrowserStorageProvider,
  createBudgetedStorageHost,
  createNamespacedStorageHost,
  createWindowStorageHost,
} from "../../../../runtime/ts/src/browser/provider-storage"
import { startBrowserProviders } from "../../../../runtime/ts/src/browser/providers"
import * as effectRuntime from "../../../../runtime/ts/src/effect"
import { processHashSeed } from "../../../../runtime/ts/src/hash"
import type { ProviderPackageEntry } from "../../../../runtime/ts/src/provider-package"
import {
  renderDebug,
  renderShow,
  type Show,
  unitDebug,
} from "../../../../runtime/ts/src/show"
import type {
  DisplayDictionaryContract,
  EntryContract,
} from "../compiler/types"
import type { ForeignRuntimeBundle } from "./foreign-runtime"
import { checkpointTransformer } from "./execution-budget"
import { runtimeModules } from "./runtime-modules"

type ModuleExports = Record<string, unknown>

const playgroundStorageNamespace = "seseragi:playground:application:"
const playgroundStorageBudgetCodeUnits = 256 * 1024

export type GeneratedProjectModule = Readonly<{
  readonly path: string
  readonly typescript: string
}>

export type ExecutionResult = {
  readonly stdout: string
  readonly debug: string
}

export type BrowserExecutionCompletion =
  | Readonly<{ readonly kind: "completed"; readonly result: ExecutionResult }>
  | Readonly<{ readonly kind: "cancelled" }>

export type BrowserExecution = Readonly<{
  readonly completion: Promise<BrowserExecutionCompletion>
  readonly cancel: () => Promise<void>
}>

export type BrowserExecutionOptions = Readonly<{
  readonly foreignRuntime?: ForeignRuntimeBundle
  readonly domDocument?: Document
  readonly onDomMounted?: () => void
  readonly storageProvider?: ProviderPackageEntry
  readonly navigationProvider?: ProviderPackageEntry
  readonly maxOutputCodeUnits?: number
  readonly checkpoint?: () => void
}>

export class ProjectExecutionError extends Error {
  readonly code = "missing-entry"
  readonly entryPath: string

  constructor(entryPath: string) {
    super(`generated project omitted entry module: ${entryPath}`)
    this.name = "ProjectExecutionError"
    this.entryPath = entryPath
  }
}

export async function executeGeneratedModule(
  typescript: string,
  entry: EntryContract,
  input = "",
  options: BrowserExecutionOptions = {}
): Promise<ExecutionResult> {
  const execution = await startGeneratedModule(
    typescript,
    entry,
    input,
    options
  )
  return completedExecutionResult(await execution.completion)
}

export async function startGeneratedModule(
  typescript: string,
  entry: EntryContract,
  input = "",
  options: BrowserExecutionOptions = {}
): Promise<BrowserExecution> {
  processHashSeed()
  const generated = await evaluate(typescript, options.checkpoint)
  return startEvaluatedModule(generated, entry, input, options, (specifier) =>
    specifier === "./main.ts"
      ? generated
      : ((Object.hasOwn(runtimeModules, specifier)
          ? runtimeModules[specifier]
          : undefined) as ModuleExports | undefined)
  )
}

export async function executeGeneratedProject(
  modules: readonly GeneratedProjectModule[],
  entryPath: string,
  entry: EntryContract,
  input = "",
  options: BrowserExecutionOptions = {}
): Promise<ExecutionResult> {
  const execution = await startGeneratedProject(
    modules,
    entryPath,
    entry,
    input,
    options
  )
  return completedExecutionResult(await execution.completion)
}

export async function startGeneratedProject(
  modules: readonly GeneratedProjectModule[],
  entryPath: string,
  entry: EntryContract,
  input = "",
  options: BrowserExecutionOptions = {}
): Promise<BrowserExecution> {
  processHashSeed()
  const entryModulePath = generatedModulePath(entryPath)
  const evaluated = await evaluateProject(
    modules,
    entryPath,
    options.foreignRuntime,
    options.checkpoint
  )
  const generated = evaluated.modules.get(entryModulePath)
  if (generated === undefined) {
    throw new ProjectExecutionError(entryPath)
  }
  return startEvaluatedModule(generated, entry, input, options, (specifier) => {
    if (specifier === "./main.ts") return generated
    const runtime = Object.hasOwn(runtimeModules, specifier)
      ? runtimeModules[specifier]
      : undefined
    if (runtime !== undefined) return runtime as ModuleExports
    if (!specifier.startsWith(".")) return undefined
    return evaluated.modules.get(generatedContractPath(specifier))
  })
}

async function startEvaluatedModule(
  generated: ModuleExports,
  entry: EntryContract,
  input: string,
  options: BrowserExecutionOptions,
  resolveModule: (specifier: string) => ModuleExports | undefined
): Promise<BrowserExecution> {
  const effectExecution = effectRuntime.createEffectExecution()
  const main = generated.main
  if (typeof main !== "function") {
    throw new Error("generated module does not export main")
  }
  const requiresDom = entry.environment.some(
    (binding) => binding.service === "dom"
  )
  const requiresNavigation = entry.environment.some(
    (binding) => binding.service === "navigation"
  )
  const requiresStorage = entry.environment.some(
    (binding) => binding.service === "storage"
  )
  if (requiresDom && options.domDocument === undefined) {
    throw new Error("program requires an interactive HTML preview")
  }
  if (
    ((requiresNavigation && options.navigationProvider === undefined) ||
      (requiresStorage && options.storageProvider === undefined)) &&
    options.domDocument?.defaultView == null
  ) {
    throw new Error(
      "program browser capabilities require an interactive HTML preview"
    )
  }
  const providerRuntime = await startBrowserProviders(
    entry.providers ?? [],
    async (specifier) => {
      if (
        specifier === "seseragi/runtime-browser/navigation" &&
        (options.navigationProvider !== undefined ||
          options.domDocument?.defaultView != null)
      ) {
        return {
          provider:
            options.navigationProvider ??
            createBrowserNavigationProvider(
              createWindowNavigationHost(options.domDocument!.defaultView!)
            ),
        }
      }
      if (
        specifier === "seseragi/runtime-browser/storage" &&
        (options.storageProvider !== undefined ||
          options.domDocument?.defaultView != null)
      ) {
        return {
          provider:
            options.storageProvider ??
            createBrowserStorageProvider(
              createBudgetedStorageHost(
                createNamespacedStorageHost(
                  createWindowStorageHost(options.domDocument!.defaultView!),
                  playgroundStorageNamespace
                ),
                playgroundStorageBudgetCodeUnits,
                playgroundStorageNamespace.length
              )
            ),
        }
      }
      const module = resolveModule(specifier)
      if (module === undefined) {
        throw new Error(`unsupported browser provider module: ${specifier}`)
      }
      // IIFE bundlers represent trusted ESM namespaces with accessors. Give
      // the provider loader a data-property snapshot without weakening its
      // checks on provider entry metadata or operation descriptors.
      return Object.hasOwn(runtimeModules, specifier)
        ? Object.freeze({ ...module })
        : module
    }
  )
  let stdout = ""
  let browserDom: BrowserDom | undefined
  let environment: Record<string, unknown>
  try {
    if (requiresDom && options.domDocument !== undefined) {
      browserDom = createBrowserDom(options.domDocument, () => {
        if (!effectExecution.context.cancelled) {
          options.onDomMounted?.()
        }
      })
    }
    environment = createBrowserEnvironment(
      entry.environment,
      input,
      (value) => {
        if (!effectExecution.context.cancelled) {
          if (
            options.maxOutputCodeUnits !== undefined &&
            stdout.length + value.length > options.maxOutputCodeUnits
          )
            throw new Error(
              `Program output limit exceeded (${options.maxOutputCodeUnits} UTF-16 code units)`
            )
          stdout += value
        }
      },
      browserDom?.service,
      effectExecution.context,
      providerRuntime.services
    )
  } catch (error) {
    await browserDom?.dispose()
    await providerRuntime.shutdown()
    throw error
  }

  let domDisposal: Promise<void> | undefined
  const disposeDom = (): Promise<void> => {
    domDisposal ??= disposeBrowserDom(
      browserDom,
      () => effectExecution.context.cancelled
    )
    return domDisposal
  }
  const completion = effectRuntime
    .run(
      main(undefined) as effectRuntime.Effect<unknown, unknown, unknown>,
      environment,
      effectExecution.context
    )
    .then((result) => {
      if (result.kind === "failure") {
        throw new Error(renderFailure(entry, resolveModule, result.error))
      }
      return Object.freeze({
        kind: "completed" as const,
        result: Object.freeze({
          stdout: stdout.trimEnd(),
          debug: renderDebug(unitDebug, result.value as undefined, {
            layout: "auto",
          }),
        }),
      })
    })
    .catch((error: unknown) => {
      if (effectRuntime.isEffectCancellation(error)) {
        return Object.freeze({ kind: "cancelled" as const })
      }
      throw error
    })
    .finally(async () => {
      await disposeDom()
      await providerRuntime.shutdown()
    })
  let cancellation: Promise<void> | undefined
  return Object.freeze({
    completion,
    cancel: () => {
      if (cancellation !== undefined) return cancellation
      cancellation = effectExecution.cancel().then(async () => {
        await completion.then(
          () => undefined,
          () => undefined
        )
      })
      return cancellation
    },
  })
}

function completedExecutionResult(
  completion: BrowserExecutionCompletion
): ExecutionResult {
  if (completion.kind === "completed") return completion.result
  throw new Error("browser execution was cancelled")
}

async function disposeBrowserDom(
  browserDom: BrowserDom | undefined,
  isCancelled: () => boolean
): Promise<void> {
  try {
    await browserDom?.dispose()
  } catch (error) {
    if (!isCancelled()) throw error
  }
}

async function evaluate(
  source: string,
  checkpoint?: () => void
): Promise<ModuleExports> {
  const guardName =
    checkpoint === undefined
      ? undefined
      : `__checkpoint_${crypto.randomUUID().replaceAll("-", "")}`
  const javascript = await transpile(source, undefined, guardName)
  const module = { exports: {} as ModuleExports }
  const requireRuntime = (specifier: string): unknown => {
    const runtime = Object.hasOwn(runtimeModules, specifier)
      ? runtimeModules[specifier]
      : undefined
    if (runtime === undefined) {
      throw new Error(`unsupported playground runtime module: ${specifier}`)
    }
    return runtime
  }
  executeCommonJs(javascript, module, requireRuntime, checkpoint, guardName)
  return module.exports
}

async function evaluateProject(
  modules: readonly GeneratedProjectModule[],
  entryPath: string,
  foreignRuntime?: ForeignRuntimeBundle,
  checkpoint?: () => void
): Promise<Readonly<{ modules: ReadonlyMap<string, ModuleExports> }>> {
  const guardName =
    checkpoint === undefined
      ? undefined
      : `__checkpoint_${crypto.randomUUID().replaceAll("-", "")}`
  const sources = new Map<string, string>()
  for (const module of modules) {
    const path = generatedModulePath(module.path)
    if (
      foreignRuntime?.generatedPaths !== undefined &&
      !foreignRuntime.generatedPaths.includes(path)
    )
      continue
    if (sources.has(path)) {
      throw new Error(`duplicate generated project module: ${module.path}`)
    }
    sources.set(
      path,
      await transpile(
        module.typescript,
        (specifier) => {
          const resolved = foreignRuntime?.roots[path]?.[specifier]
          const artifact =
            resolved === undefined
              ? undefined
              : foreignRuntime?.modules[resolved]
          if (artifact === undefined)
            throw new Error(
              `SES-F0203: unresolved browser runtime identity: ${specifier}`
            )
          return `${artifact.exactIdentity}#${artifact.contentDigest}#${foreignRuntime?.snapshotDigest}`
        },
        guardName
      )
    )
  }

  for (const [path, module] of Object.entries(foreignRuntime?.modules ?? {})) {
    if (sources.has(path))
      throw new Error(`foreign runtime shadows generated module: ${path}`)
    sources.set(
      path,
      await transpile(
        module.source,
        (specifier) => {
          const resolved = module.imports[specifier]
          const artifact =
            resolved === undefined
              ? undefined
              : foreignRuntime?.modules[resolved]
          if (artifact === undefined)
            throw new Error(
              `SES-F0203: unresolved browser runtime identity: ${specifier}`
            )
          return `${artifact.exactIdentity}#${artifact.contentDigest}#${foreignRuntime?.snapshotDigest}`
        },
        guardName
      )
    )
  }

  const evaluated = new Map<string, ModuleExports>()
  const evaluating = new Set<string>()
  const load = (path: string): ModuleExports => {
    const cached = evaluated.get(path)
    if (cached !== undefined) return cached
    const source = sources.get(path)
    if (source === undefined) {
      throw new Error(`generated project module not found: ${path}`)
    }
    if (evaluating.has(path)) {
      throw new Error(`generated project import cycle: ${path}`)
    }
    evaluating.add(path)
    const module = { exports: {} as ModuleExports }
    evaluated.set(path, module.exports)
    try {
      executeCommonJs(
        source,
        module,
        (specifier) => {
          const runtime = Object.hasOwn(runtimeModules, specifier)
            ? runtimeModules[specifier]
            : undefined
          if (runtime !== undefined) return runtime
          const foreignPath =
            foreignRuntime?.modules[path]?.imports[specifier] ??
            foreignRuntime?.roots[path]?.[specifier]
          if (foreignPath !== undefined) return load(foreignPath)
          if (!specifier.startsWith(".")) {
            throw new Error(
              `unsupported playground runtime module: ${specifier}`
            )
          }
          return load(resolveGeneratedSpecifier(path, specifier))
        },
        checkpoint,
        guardName
      )
      evaluated.set(path, module.exports)
      return module.exports
    } finally {
      evaluating.delete(path)
    }
  }

  const entryModulePath = generatedModulePath(entryPath)
  if (!sources.has(entryModulePath)) {
    throw new ProjectExecutionError(entryPath)
  }
  load(entryModulePath)
  return Object.freeze({ modules: evaluated })
}

async function transpile(
  source: string,
  resolveIdentity?: (specifier: string) => string,
  guardName?: string
): Promise<string> {
  const ts = await import("typescript")
  const javascript = ts.transpileModule(source, {
    transformers:
      resolveIdentity === undefined
        ? undefined
        : {
            before: [
              (context) => {
                const visit = (
                  node: import("typescript").Node
                ): import("typescript").VisitResult<
                  import("typescript").Node
                > => {
                  if (
                    ts.isCallExpression(node) &&
                    ts.isPropertyAccessExpression(node.expression) &&
                    node.expression.name.text === "resolve" &&
                    ts.isMetaProperty(node.expression.expression) &&
                    node.expression.expression.keywordToken ===
                      ts.SyntaxKind.ImportKeyword
                  ) {
                    const argument = node.arguments[0]
                    if (
                      node.arguments.length !== 1 ||
                      argument === undefined ||
                      !ts.isStringLiteral(argument)
                    )
                      throw new Error(
                        "SES-F0203: computed import.meta.resolve has unresolved runtime identity"
                      )
                    return ts.factory.createStringLiteral(
                      resolveIdentity(argument.text)
                    )
                  }
                  return ts.visitEachChild(node, visit, context)
                }
                return (file) =>
                  ts.visitNode(file, visit) as import("typescript").SourceFile
              },
            ],
          },
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
      strict: true,
    },
  }).outputText
  if (guardName === undefined) return javascript
  const tree = ts.createSourceFile(
    "program.js",
    javascript,
    ts.ScriptTarget.ES2022,
    true,
    ts.ScriptKind.JS
  )
  const transformed = ts.transform(tree, [checkpointTransformer(ts, guardName)])
  try {
    return ts.createPrinter().printFile(transformed.transformed[0]!)
  } finally {
    transformed.dispose()
  }
}

function executeCommonJs(
  javascript: string,
  module: { exports: ModuleExports },
  requireModule: (specifier: string) => unknown,
  checkpoint?: () => void,
  guardName?: string
): void {
  const execute =
    guardName === undefined
      ? new Function("require", "module", "exports", javascript)
      : new Function("require", "module", "exports", guardName, javascript)
  execute(requireModule, module, module.exports, checkpoint)
}

function generatedModulePath(sourcePath: string): string {
  return sourcePath.endsWith(".ssrg")
    ? `${sourcePath.slice(0, -5)}.js`
    : sourcePath
}

function resolveGeneratedSpecifier(
  fromPath: string,
  specifier: string
): string {
  const parts = fromPath.split("/")
  parts.pop()
  for (const part of specifier.split("/")) {
    if (part === "" || part === ".") continue
    if (part === "..") {
      if (parts.length === 0) {
        throw new Error(`generated import escapes project: ${specifier}`)
      }
      parts.pop()
      continue
    }
    parts.push(part)
  }
  const path = parts.join("/")
  return path.endsWith(".ts") ? `${path.slice(0, -3)}.js` : path
}

function generatedContractPath(specifier: string): string {
  const parts: string[] = []
  for (const part of specifier.split("/")) {
    if (part === "" || part === ".") continue
    if (part === "..") {
      throw new Error(`generated contract escapes project: ${specifier}`)
    }
    parts.push(part)
  }
  const path = parts.join("/")
  return path.endsWith(".ts") ? `${path.slice(0, -3)}.js` : path
}

function renderFailure(
  entry: EntryContract,
  resolveModule: (specifier: string) => ModuleExports | undefined,
  error: unknown
): string {
  const renderer = entry.failureRenderer
  if (renderer.kind === "never") return "seseragi: unreachable typed failure"
  const dictionary = displayDictionary(resolveModule, renderer) as
    | Show<unknown>
    | undefined
  if (dictionary === undefined || typeof dictionary.show !== "function") {
    throw new Error("Show dictionary returned a non-string value")
  }
  return renderShow(dictionary, error, { layout: "compact" })
}

function displayDictionary(
  resolveModule: (specifier: string) => ModuleExports | undefined,
  contract: DisplayDictionaryContract
): unknown {
  const source = resolveModule(contract.module)
  const binding = source?.[contract.export]
  const arguments_ = contract.arguments ?? []
  if (arguments_.length === 0) {
    return binding
  }
  if (typeof binding !== "function") {
    throw new Error("display dictionary factory is not callable")
  }
  return binding(
    ...arguments_.map((argument) => displayDictionary(resolveModule, argument))
  )
}

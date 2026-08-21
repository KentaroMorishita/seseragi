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
  createWindowStorageHost,
} from "../../../../runtime/ts/src/browser/provider-storage"
import { startBrowserProviders } from "../../../../runtime/ts/src/browser/providers"
import * as effectRuntime from "../../../../runtime/ts/src/effect"
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
import { runtimeModules } from "./runtime-modules"

type ModuleExports = Record<string, unknown>

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
  readonly domDocument?: Document
  readonly onDomMounted?: () => void
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
  const generated = await evaluate(typescript)
  return startEvaluatedModule(generated, entry, input, options, (specifier) =>
    specifier === "./main.ts"
      ? generated
      : (runtimeModules[specifier] as ModuleExports | undefined)
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
  const entryModulePath = generatedModulePath(entryPath)
  const evaluated = await evaluateProject(modules, entryPath)
  const generated = evaluated.modules.get(entryModulePath)
  if (generated === undefined) {
    throw new ProjectExecutionError(entryPath)
  }
  return startEvaluatedModule(generated, entry, input, options, (specifier) => {
    if (specifier === "./main.ts") return generated
    const runtime = runtimeModules[specifier]
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
    (requiresNavigation || requiresStorage) &&
    (options.domDocument === undefined ||
      options.domDocument.defaultView === null)
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
        options.domDocument?.defaultView !== null &&
        options.domDocument?.defaultView !== undefined
      ) {
        return {
          provider: createBrowserNavigationProvider(
            createWindowNavigationHost(options.domDocument.defaultView)
          ),
        }
      }
      if (
        specifier === "seseragi/runtime-browser/storage" &&
        options.domDocument?.defaultView !== null &&
        options.domDocument?.defaultView !== undefined
      ) {
        return {
          provider: createBrowserStorageProvider(
            createWindowStorageHost(options.domDocument.defaultView)
          ),
        }
      }
      const module = resolveModule(specifier)
      if (module === undefined) {
        throw new Error(`unsupported browser provider module: ${specifier}`)
      }
      return module
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
        if (!effectExecution.context.cancelled) stdout += value
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
  const unregisterDomDisposal =
    browserDom === undefined
      ? undefined
      : effectExecution.context.onCancel(disposeDom)
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
      unregisterDomDisposal?.()
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

async function evaluate(source: string): Promise<ModuleExports> {
  const javascript = await transpile(source)
  const module = { exports: {} as ModuleExports }
  const requireRuntime = (specifier: string): unknown => {
    const runtime = runtimeModules[specifier]
    if (runtime === undefined) {
      throw new Error(`unsupported playground runtime module: ${specifier}`)
    }
    return runtime
  }
  executeCommonJs(javascript, module, requireRuntime)
  return module.exports
}

async function evaluateProject(
  modules: readonly GeneratedProjectModule[],
  entryPath: string
): Promise<Readonly<{ modules: ReadonlyMap<string, ModuleExports> }>> {
  const sources = new Map<string, string>()
  for (const module of modules) {
    const path = generatedModulePath(module.path)
    if (sources.has(path)) {
      throw new Error(`duplicate generated project module: ${module.path}`)
    }
    sources.set(path, await transpile(module.typescript))
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
      executeCommonJs(source, module, (specifier) => {
        const runtime = runtimeModules[specifier]
        if (runtime !== undefined) return runtime
        if (!specifier.startsWith(".")) {
          throw new Error(`unsupported playground runtime module: ${specifier}`)
        }
        return load(resolveGeneratedSpecifier(path, specifier))
      })
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

async function transpile(source: string): Promise<string> {
  const ts = await import("typescript")
  return ts.transpileModule(source, {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
      strict: true,
    },
  }).outputText
}

function executeCommonJs(
  javascript: string,
  module: { exports: ModuleExports },
  requireModule: (specifier: string) => unknown
): void {
  const execute = new Function("require", "module", "exports", javascript)
  execute(requireModule, module, module.exports)
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

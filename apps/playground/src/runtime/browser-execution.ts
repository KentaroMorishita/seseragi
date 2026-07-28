import { createBrowserEnvironment } from "../../../../runtime/ts/src/browser/host"
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
import { type BrowserDom, createBrowserDom } from "./browser-dom"
import { runtimeModules } from "./runtime-modules"

type ModuleExports = Record<string, unknown>

export type ExecutionResult = {
  readonly stdout: string
  readonly debug: string
}

export type BrowserExecution = Readonly<{
  readonly completion: Promise<ExecutionResult>
  readonly cancel: () => Promise<void>
}>

export type BrowserExecutionOptions = Readonly<{
  readonly domDocument?: Document
  readonly onDomMounted?: () => void
}>

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
  return execution.completion
}

export async function startGeneratedModule(
  typescript: string,
  entry: EntryContract,
  input = "",
  options: BrowserExecutionOptions = {}
): Promise<BrowserExecution> {
  let stdout = ""
  let browserDom: BrowserDom | undefined
  if (entry.environment.some((binding) => binding.service === "dom")) {
    if (options.domDocument === undefined) {
      throw new Error("program requires an interactive HTML preview")
    }
    browserDom = createBrowserDom(
      options.domDocument,
      options.onDomMounted ?? (() => {})
    )
  }
  const environment = createBrowserEnvironment(
    entry.environment,
    input,
    (value) => {
      stdout += value
    },
    browserDom?.service
  )
  const generated = await evaluate(typescript)
  const main = generated.main
  if (typeof main !== "function") {
    throw new Error("generated module does not export main")
  }

  const completion = effectRuntime
    .run(
      main(undefined) as effectRuntime.Effect<unknown, unknown, unknown>,
      environment
    )
    .then((result) => {
      if (result.kind === "failure") {
        throw new Error(renderFailure(entry, generated, result.error))
      }
      return {
        stdout: stdout.trimEnd(),
        debug: renderDebug(unitDebug, result.value as undefined, {
          layout: "auto",
        }),
      }
    })
    .finally(() => browserDom?.dispose())
  return Object.freeze({
    completion,
    cancel: () => browserDom?.dispose() ?? Promise.resolve(),
  })
}

async function evaluate(source: string): Promise<ModuleExports> {
  const ts = await import("typescript")
  const javascript = ts.transpileModule(source, {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
      strict: true,
    },
  }).outputText
  const module = { exports: {} as ModuleExports }
  const requireRuntime = (specifier: string): unknown => {
    const runtime = runtimeModules[specifier]
    if (runtime === undefined) {
      throw new Error(`unsupported playground runtime module: ${specifier}`)
    }
    return runtime
  }
  const execute = new Function("require", "module", "exports", javascript)
  execute(requireRuntime, module, module.exports)
  return module.exports
}

function renderFailure(
  entry: EntryContract,
  generated: ModuleExports,
  error: unknown
): string {
  const renderer = entry.failureRenderer
  if (renderer.kind === "never") return "seseragi: unreachable typed failure"
  const dictionary = displayDictionary(generated, renderer) as
    | Show<unknown>
    | undefined
  if (dictionary === undefined || typeof dictionary.show !== "function") {
    throw new Error("Show dictionary returned a non-string value")
  }
  return renderShow(dictionary, error, { layout: "compact" })
}

function displayDictionary(
  generated: ModuleExports,
  contract: DisplayDictionaryContract
): unknown {
  const source =
    contract.module === "./main.ts"
      ? generated
      : (runtimeModules[contract.module] as ModuleExports | undefined)
  const binding = source?.[contract.export]
  const arguments_ = contract.arguments ?? []
  if (arguments_.length === 0) {
    return binding
  }
  if (typeof binding !== "function") {
    throw new Error("display dictionary factory is not callable")
  }
  return binding(
    ...arguments_.map((argument) => displayDictionary(generated, argument))
  )
}

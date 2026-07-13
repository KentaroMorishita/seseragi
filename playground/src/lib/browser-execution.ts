import * as effectRuntime from "../../../runtime/ts/src/effect"
import { createBrowserEnvironment } from "../../../runtime/ts/src/browser/host"
import * as ts from "typescript"
import { runtimeModules } from "./runtime-modules"
import type { EntryContract } from "./playground-types"

type ModuleExports = Record<string, unknown>

export async function executeGeneratedModule(
  typescript: string,
  entry: EntryContract,
  input = ""
): Promise<string> {
  let output = ""
  const environment = createBrowserEnvironment(
    entry.environment,
    input,
    (value) => {
      output += value
    }
  )
  const exports = evaluate(typescript)
  const main = exports.main
  if (typeof main !== "function") {
    throw new Error("generated module does not export main")
  }

  const result = await effectRuntime.run(
    main(undefined) as effectRuntime.Effect<unknown, unknown, unknown>,
    environment
  )
  if (result.kind === "failure") {
    throw new Error(renderFailure(entry, exports, result.error))
  }
  return output.trimEnd() || "Program executed successfully (no output)"
}

function evaluate(source: string): ModuleExports {
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
  const source =
    renderer.module === "./main.ts"
      ? generated
      : (runtimeModules[renderer.module] as ModuleExports | undefined)
  const dictionary = source?.[renderer.export] as
    | { readonly show?: (value: unknown) => unknown }
    | undefined
  const message = dictionary?.show?.(error)
  if (typeof message !== "string") {
    throw new Error("Show dictionary returned a non-string value")
  }
  return message
}

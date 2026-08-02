import { createCapturedConsole } from "./console"
import { createTextStdin } from "./stdin"
import type { Dom } from "../dom"
import { attachEffectContext, type EffectContext } from "../effect"

export type HostService = "console" | "stdin" | "dom"

export type EnvironmentBinding = {
  readonly field: string
  readonly service: HostService
}

export function createBrowserEnvironment(
  bindings: readonly EnvironmentBinding[],
  input: string,
  write: (value: string) => void,
  dom: Dom | undefined,
  context: EffectContext
): Record<string, unknown> {
  const environment: Record<string, unknown> = {}
  for (const binding of bindings) {
    switch (binding.service) {
      case "console":
        environment[binding.field] = createCapturedConsole(write)
        break
      case "stdin":
        environment[binding.field] = createTextStdin(input)
        break
      case "dom":
        if (dom === undefined) {
          throw new Error("program requires a browser DOM host")
        }
        environment[binding.field] = dom
        break
    }
  }
  return attachEffectContext(environment, context)
}

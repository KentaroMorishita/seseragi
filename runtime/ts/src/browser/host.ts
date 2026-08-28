import type { Dom } from "../dom"
import { attachEffectContext, type EffectContext } from "../effect"
import { createCapturedConsole } from "./console"
import { createCapturedLogger } from "./logger"
import type { BrowserProviderServices } from "./providers"
import { createTextStdin } from "./stdin"

export type HostService =
  | "console"
  | "logger"
  | "stdin"
  | "dom"
  | "clock"
  | "httpClient"
  | "webSocketClient"
  | "navigation"
  | "storage"
  | "random"
  | "entropy"

export type EnvironmentBinding = {
  readonly field: string
  readonly service: HostService
}

export function createBrowserEnvironment(
  bindings: readonly EnvironmentBinding[],
  input: string,
  write: (value: string) => void,
  dom: Dom | undefined,
  context: EffectContext,
  providers: BrowserProviderServices = {}
): Record<string, unknown> {
  const environment: Record<string, unknown> = {}
  for (const binding of bindings) {
    switch (binding.service) {
      case "console":
        environment[binding.field] = createCapturedConsole(write)
        break
      case "logger":
        environment[binding.field] = createCapturedLogger(write)
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
      case "clock":
        if (providers.clock === undefined) {
          throw new Error(
            "program requires the resolved browser Clock provider"
          )
        }
        environment[binding.field] = providers.clock
        break
      case "httpClient":
        if (providers.httpClient === undefined) {
          throw new Error(
            "program requires the resolved browser HTTP client provider"
          )
        }
        environment[binding.field] = providers.httpClient
        break
      case "webSocketClient":
        if (providers.webSocketClient === undefined) {
          throw new Error(
            "program requires the resolved browser WebSocket client provider"
          )
        }
        environment[binding.field] = providers.webSocketClient
        break
      case "navigation":
        if (providers.navigation === undefined) {
          throw new Error(
            "program requires the resolved browser Navigation provider"
          )
        }
        environment[binding.field] = providers.navigation
        break
      case "storage":
        if (providers.storage === undefined) {
          throw new Error(
            "program requires the resolved browser Storage provider"
          )
        }
        environment[binding.field] = providers.storage
        break
      case "random":
        if (providers.random === undefined) {
          throw new Error(
            "program requires the resolved browser Random provider"
          )
        }
        environment[binding.field] = providers.random
        break
      case "entropy":
        if (providers.entropy === undefined) {
          throw new Error(
            "program requires the resolved browser Entropy provider"
          )
        }
        environment[binding.field] = providers.entropy
        break
    }
  }
  return attachEffectContext(environment, context)
}

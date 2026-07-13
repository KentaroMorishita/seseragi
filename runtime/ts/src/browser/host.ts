import { createCapturedConsole } from "./console"
import { createTextStdin } from "./stdin"

export type HostService = "console" | "stdin"

export type EnvironmentBinding = {
  readonly field: string
  readonly service: HostService
}

export function createBrowserEnvironment(
  bindings: readonly EnvironmentBinding[],
  input: string,
  write: (value: string) => void
): Record<string, unknown> {
  const environment: Record<string, unknown> = {}
  for (const binding of bindings) {
    environment[binding.field] =
      binding.service === "console"
        ? createCapturedConsole(write)
        : createTextStdin(input)
  }
  return environment
}

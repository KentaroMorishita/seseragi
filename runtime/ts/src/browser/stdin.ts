import { serviceSuccess } from "../service"
import type { Stdin } from "../stdin-service"
import { Just, Nothing } from "../sum"

export type { Stdin, StdinEnvironment, StdinError } from "../stdin-service"
export { readLine } from "../stdin-service"

export function createTextStdin(input: string): Stdin {
  const normalized = input.replaceAll("\r\n", "\n")
  const lines = normalized === "" ? [] : normalized.split("\n")
  if (lines.at(-1) === "") lines.pop()
  let cursor = 0
  return {
    readLine() {
      if (cursor >= lines.length) return serviceSuccess(Nothing)
      const line = lines[cursor]
      cursor += 1
      return serviceSuccess(Just(line))
    },
  }
}

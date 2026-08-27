import type { Console } from "../console-service"
import { type Unit, unit } from "../effect"
import { serviceSuccess } from "../service"

export type {
  Console,
  ConsoleEnvironment,
  ConsoleError,
} from "../console-service"
export {
  error,
  errorLine,
  flush,
  print,
  println,
  printValue,
} from "../console-service"

export function createCapturedConsole(write: (value: string) => void): Console {
  const succeed = (value: string) => {
    write(value)
    return serviceSuccess<Unit>(unit)
  }
  return {
    print: succeed,
    println(value) {
      return succeed(`${value}\n`)
    },
    error: succeed,
    errorLine(value) {
      return succeed(`${value}\n`)
    },
    flush() {
      return serviceSuccess<Unit>(unit)
    },
  }
}

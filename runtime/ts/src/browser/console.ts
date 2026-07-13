import { unit, type Unit } from "../effect"
import { serviceSuccess } from "../service"
import type { Console } from "../console-service"

export type {
  Console,
  ConsoleEnvironment,
  ConsoleError,
} from "../console-service"
export { print, println } from "../console-service"

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
  }
}

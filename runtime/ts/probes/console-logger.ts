import {
  createCapturedConsole,
  error,
  errorLine,
  flush,
  print,
  println,
  printValue,
} from "../src/browser/console"
import { createCapturedLogger } from "../src/browser/logger"
import { run, unit } from "../src/effect"
import { fromArray } from "../src/list"
import { log, LogBool, LogInfo, LogInt, LogString } from "../src/logger"
import { serviceFailure, serviceSuccess } from "../src/service"
import { intShow } from "../src/show"

function require(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

const consoleWrites: string[] = []
const loggerWrites: string[] = []
const consoleService = createCapturedConsole((value) =>
  consoleWrites.push(value)
)
const loggerService = createCapturedLogger((value) => loggerWrites.push(value))

const consoleEffects = [
  print("out"),
  println(" line"),
  printValue(42, intShow),
  error(" err"),
  errorLine(" line"),
  flush(),
] as const
const loggerEffect = log({
  level: LogInfo,
  message: "structured",
  fields: fromArray([
    ["first", LogInt(1)] as const,
    ["second", LogString("two")] as const,
    ["third", LogBool(true)] as const,
  ]),
})
require(consoleWrites.length === 0 &&
  loggerWrites.length === 0, "console or logger effect was not cold")
for (const effect of consoleEffects) {
  const result = await run(effect, { console: consoleService })
  require(result.kind === "success", "captured console operation failed")
}
const loggerResult = await run(loggerEffect, { logger: loggerService })
require(loggerResult.kind === "success", "captured logger operation failed")
require(JSON.stringify(consoleWrites) ===
  JSON.stringify([
    "out",
    " line\n",
    "42",
    " err",
    " line\n",
  ]), "console routing or printValue rendering changed")
require(loggerWrites.length === 1 &&
  loggerWrites[0] ===
    '{"level":"info","message":"structured","fields":[["first",1],["second","two"],["third",true]]}\n', "logger event shape or field ordering changed")
require(consoleWrites.every(
  (value) => !value.includes("structured")
), "logger was routed through Console")

const typedConsole = {
  print() {
    return serviceFailure({ kind: "console-error" as const, message: "typed" })
  },
  println() {
    return serviceSuccess(unit)
  },
  error() {
    return serviceSuccess(unit)
  },
  errorLine() {
    return serviceSuccess(unit)
  },
  flush() {
    return serviceSuccess(unit)
  },
}
const typedFailure = await run(print("typed"), { console: typedConsole })
require(typedFailure.kind === "failure" &&
  typedFailure.error.kind === "console-error" &&
  typedFailure.error.message ===
    "typed", "ConsoleError escaped the typed failure channel")

const loggerFailure = await run(
  log({
    level: LogInfo,
    message: "typed",
    fields: fromArray([]),
  }),
  {
    logger: {
      log() {
        return serviceFailure({ kind: "log-error" as const, message: "typed" })
      },
    },
  }
)
require(loggerFailure.kind === "failure" &&
  loggerFailure.error.kind === "log-error" &&
  loggerFailure.error.message ===
    "typed", "LogError escaped the typed failure channel")

const rawDefect = new Error("raw console defect")
let rawDefectPreserved = false
try {
  await run(print("defect"), {
    console: {
      ...consoleService,
      print() {
        throw rawDefect
      },
    },
  })
} catch (error) {
  rawDefectPreserved = error === rawDefect
}
require(rawDefectPreserved, "raw Console defect became a typed failure")

console.log("console and logger runtime probe passed")

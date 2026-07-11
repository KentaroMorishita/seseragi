import type { ConsoleError } from "../src/console"
import {
  consoleErrorShow,
  type Show,
  stdinErrorShow,
  stringShow,
} from "../src/show"
import type { StdinError } from "../src/stdin"

function assertEqual(actual: string, expected: string): void {
  if (actual !== expected) {
    throw new Error(
      `expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`
    )
  }
}

const dictionaries: readonly Show<unknown>[] = [
  stringShow as Show<unknown>,
  consoleErrorShow as Show<unknown>,
  stdinErrorShow as Show<unknown>,
]
if (dictionaries.some((dictionary) => typeof dictionary.show !== "function")) {
  throw new Error("a standard Show dictionary has an invalid runtime shape")
}

assertEqual(stringShow.show("hello\nworld"), "hello\nworld")

const consoleError: ConsoleError = {
  kind: "console-error",
  message: "broken pipe",
}
assertEqual(consoleErrorShow.show(consoleError), "ConsoleError: broken pipe")

const stdinCases: ReadonlyArray<readonly [StdinError, string]> = [
  [{ tag: "StdinUnavailable" }, "StdinUnavailable"],
  [{ tag: "StdinReadFailure" }, "StdinReadFailure"],
  [{ tag: "ConcurrentStdinRead" }, "ConcurrentStdinRead"],
  [
    { tag: "InvalidStdinUtf8", value: { offset: 12n } },
    "InvalidStdinUtf8 { offset: 12 }",
  ],
  [
    { tag: "StdinLineTooLong", value: { limitBytes: 1024n } },
    "StdinLineTooLong { limitBytes: 1024 }",
  ],
  [{ tag: "StdinPositionOverflow" }, "StdinPositionOverflow"],
]
for (const [error, expected] of stdinCases) {
  assertEqual(stdinErrorShow.show(error), expected)
}

process.stdout.write("show runtime probe passed\n")

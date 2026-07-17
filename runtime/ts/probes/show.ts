import type { ConsoleError } from "../src/console"
import {
  consoleErrorShow,
  intShow,
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
  intShow as Show<unknown>,
  consoleErrorShow as Show<unknown>,
  stdinErrorShow as Show<unknown>,
]
if (dictionaries.some((dictionary) => typeof dictionary.show !== "function")) {
  throw new Error("a standard Show dictionary has an invalid runtime shape")
}

assertEqual(stringShow.show("hello\nworld"), "hello\nworld")
assertEqual(intShow.show(0n), "0")
assertEqual(intShow.show(42n), "42")
assertEqual(intShow.show(-9_223_372_036_854_775_808n), "-9223372036854775808")

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

import type { ConsoleError } from "../src/console"
import { fromArray } from "../src/list"
import {
  arrayDebug,
  arrayShow,
  boolDebug,
  boolShow,
  charDebug,
  charShow,
  concat,
  consoleErrorShow,
  type Debug,
  delimited,
  eitherDebug,
  eitherShow,
  floatDebug,
  floatShow,
  indent,
  intDebug,
  intShow,
  line,
  listDebug,
  listShow,
  maybeDebug,
  maybeShow,
  neverDebug,
  neverShow,
  renderDebug,
  renderDocument,
  renderShow,
  type Show,
  stdinErrorShow,
  stringDebug,
  stringShow,
  text,
  unitDebug,
  unitShow,
} from "../src/show"
import type { StdinError } from "../src/stdin"
import { Just, Left, Nothing, Right } from "../src/sum"

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
  floatShow as Show<unknown>,
  neverShow as Show<unknown>,
  boolShow as Show<unknown>,
  unitShow as Show<unknown>,
  charShow as Show<unknown>,
  consoleErrorShow as Show<unknown>,
  stdinErrorShow as Show<unknown>,
]
if (dictionaries.some((dictionary) => typeof dictionary.show !== "function")) {
  throw new Error("a standard Show dictionary has an invalid runtime shape")
}

const debugDictionaries: readonly Debug<unknown>[] = [
  stringDebug as Debug<unknown>,
  intDebug as Debug<unknown>,
  floatDebug as Debug<unknown>,
  neverDebug as Debug<unknown>,
  boolDebug as Debug<unknown>,
  unitDebug as Debug<unknown>,
  charDebug as Debug<unknown>,
]
if (
  debugDictionaries.some((dictionary) => typeof dictionary.debug !== "function")
) {
  throw new Error("a standard Debug dictionary has an invalid runtime shape")
}

assertEqual(stringShow.show("hello\nworld"), "hello\nworld")
assertEqual(intShow.show(0n), "0")
assertEqual(intShow.show(42n), "42")
assertEqual(intShow.show(-9_223_372_036_854_775_808n), "-9223372036854775808")
assertEqual(intDebug.debug(42n), "42")
for (const [value, expected] of [
  [0, "0.0"],
  [-0, "-0.0"],
  [1.5, "1.5"],
  [1e21, "1e21"],
  [1e-7, "1e-7"],
  [Number.NaN, "NaN"],
  [Number.POSITIVE_INFINITY, "Infinity"],
  [Number.NEGATIVE_INFINITY, "-Infinity"],
] as const) {
  assertEqual(floatShow.show(value), expected)
  assertEqual(floatDebug.debug(value), expected)
}
assertEqual(boolShow.show(true), "True")
assertEqual(boolShow.show(false), "False")
assertEqual(boolDebug.debug(true), "True")
assertEqual(unitShow.show(undefined), "()")
assertEqual(unitDebug.debug(undefined), "()")
assertEqual(charShow.show("瀬"), "瀬")
assertEqual(charDebug.debug("瀬"), "'瀬'")
assertEqual(charDebug.debug("'"), "'\\''")
assertEqual(
  stringDebug.debug('line\n\t\\"\u{7f}'),
  '"line\\n\\t\\\\\\"\\u{7F}"'
)

const nested = delimited(
  "[",
  [text("first"), delimited("[", [text("child")], "]")],
  "]"
)
assertEqual(renderDocument(nested), "[first, [child]]")
assertEqual(
  renderDocument(nested, { layout: "multiline" }),
  "[\n  first,\n  [\n    child\n  ]\n]"
)
assertEqual(
  renderDocument(nested, { layout: "auto", maxWidth: 10 }),
  "[\n  first,\n  [\n    child\n  ]\n]"
)
assertEqual(
  renderDocument(delimited("(", [], ")"), { layout: "multiline" }),
  "()"
)
assertEqual(
  renderDocument(delimited("[", [text("only")], "]"), {
    layout: "multiline",
  }),
  "[\n  only\n]"
)
const structured = concat([
  text("let"),
  line,
  indent(concat([text("="), line, text("42")])),
])
assertEqual(renderDocument(structured), "let = 42")
assertEqual(renderDocument(structured, { layout: "multiline" }), "let\n=\n  42")
assertEqual(renderShow(charShow, "瀬", { layout: "multiline" }), "瀬")
assertEqual(renderDebug(charDebug, "瀬", { layout: "multiline" }), "'瀬'")

const stringsShow = arrayShow(stringShow)
const stringsDebug = arrayDebug(stringDebug)
assertEqual(stringsShow.show([]), "[]")
assertEqual(stringsShow.show(["alpha"]), "[alpha]")
assertEqual(stringsShow.show(["alpha", "beta"]), "[alpha, beta]")
assertEqual(
  stringsDebug.debug(["alpha", "line\nbreak"]),
  '["alpha", "line\\nbreak"]'
)
assertEqual(
  renderShow(stringsShow, ["alpha", "beta"], { layout: "multiline" }),
  "[\n  alpha,\n  beta\n]"
)
assertEqual(
  renderDebug(stringsDebug, ["alpha", "beta"], {
    layout: "auto",
    maxWidth: 10,
  }),
  '[\n  "alpha",\n  "beta"\n]'
)

const boolsShow = listShow(boolShow)
const stringsListDebug = listDebug(stringDebug)
assertEqual(boolsShow.show(fromArray([])), "`[]")
assertEqual(boolsShow.show(fromArray([true])), "`[True]")
assertEqual(boolsShow.show(fromArray([true, false])), "`[True, False]")
assertEqual(
  stringsListDebug.debug(fromArray(["first", "second"])),
  '`["first", "second"]'
)
assertEqual(
  renderShow(boolsShow, fromArray([true, false]), { layout: "multiline" }),
  "`[\n  True,\n  False\n]"
)

const optionalShow = maybeShow(stringShow)
const optionalDebug = maybeDebug(stringDebug)
assertEqual(maybeShow(neverShow).show(Nothing), "Nothing")
assertEqual(maybeDebug(neverDebug).debug(Nothing), "Nothing")
assertEqual(optionalShow.show(Nothing), "Nothing")
assertEqual(optionalShow.show(Just("value")), "Just value")
assertEqual(optionalDebug.debug(Just("value")), 'Just "value"')
assertEqual(
  renderDebug(optionalDebug, Just("value"), { layout: "multiline" }),
  'Just\n  "value"'
)

const resultShow = eitherShow(stringShow, boolShow)
const resultDebug = eitherDebug(stringDebug, boolDebug)
assertEqual(resultShow.show(Left("failure")), "Left failure")
assertEqual(resultShow.show(Right(true)), "Right True")
assertEqual(resultDebug.debug(Left("failure")), 'Left "failure"')
assertEqual(resultDebug.debug(Right(false)), "Right False")

const nestedDebug = arrayDebug(maybeDebug(stringDebug))
const nestedValue = [Just("alpha"), Nothing, Just("line\nbreak")]
assertEqual(
  nestedDebug.debug(nestedValue),
  '[Just "alpha", Nothing, Just "line\\nbreak"]'
)
assertEqual(
  renderDebug(nestedDebug, nestedValue, { layout: "multiline" }),
  '[\n  Just\n    "alpha",\n  Nothing,\n  Just\n    "line\\nbreak"\n]'
)

const fallbackShow: Show<string> = {
  show: (value) => `<${value}>`,
}
assertEqual(arrayShow(fallbackShow).show(["local"]), "[<local>]")

for (const invalid of ["", "ab", "\uD800"]) {
  let rejected = false
  try {
    charDebug.debug(invalid)
  } catch (error) {
    rejected = error instanceof RangeError
  }
  if (!rejected) {
    throw new Error("invalid Char runtime value was accepted")
  }
}

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

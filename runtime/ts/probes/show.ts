import type { ConsoleError } from "../src/console"
import type { DomError, DomRuntimeError } from "../src/dom"
import type { HtmlBuildError } from "../src/html"
import { fromArray } from "../src/list"
import { exclusive, inclusive } from "../src/range"
import {
  arrayDebug,
  arrayShow,
  boolDebug,
  boolShow,
  boundedDebug,
  boundedShow,
  charDebug,
  charShow,
  concat,
  consoleErrorDebug,
  consoleErrorShow,
  type Debug,
  delimited,
  displayDepthLimit,
  domErrorDebug,
  domErrorShow,
  domRuntimeErrorDebug,
  domRuntimeErrorShow,
  eitherDebug,
  eitherShow,
  floatDebug,
  floatShow,
  htmlBuildErrorDebug,
  htmlBuildErrorShow,
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
  rangeDebug,
  rangeShow,
  recordDebug,
  recordShow,
  renderDebug,
  renderDocument,
  renderShow,
  type Show,
  stdinErrorDebug,
  stdinErrorShow,
  stringDebug,
  stringShow,
  text,
  tupleDebug,
  tupleShow,
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
  domErrorShow as Show<unknown>,
  htmlBuildErrorShow as Show<unknown>,
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
  consoleErrorDebug as Debug<unknown>,
  stdinErrorDebug as Debug<unknown>,
  domErrorDebug as Debug<unknown>,
  htmlBuildErrorDebug as Debug<unknown>,
]
if (
  debugDictionaries.some((dictionary) => typeof dictionary.debug !== "function")
) {
  throw new Error("a standard Debug dictionary has an invalid runtime shape")
}

assertEqual(stringShow.show("hello\nworld"), "hello\nworld")
assertEqual(intShow.show(0), "0")
assertEqual(intShow.show(42), "42")
assertEqual(intShow.show(-9_007_199_254_740_991), "-9007199254740991")
assertEqual(intDebug.debug(42), "42")
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

const intRangeShow = rangeShow(intShow)
const intRangeDebug = rangeDebug(intDebug)
assertEqual(intRangeShow.show(exclusive(1, 5)), "1..5")
assertEqual(intRangeShow.show(exclusive(5, 5)), "5..5")
assertEqual(intRangeDebug.debug(inclusive(5, 5)), "5..=5")
assertEqual(intRangeDebug.debug(inclusive(10, 1)), "10..=1")
assertEqual(
  renderShow(intRangeShow, inclusive(1, 3), { layout: "multiline" }),
  "1..=3"
)
assertEqual(
  arrayShow(intRangeShow).show([exclusive(1, 5), inclusive(10, 1)]),
  "[1..5, 10..=1]"
)

const pairShow = tupleShow<readonly [number, string]>(intShow, stringShow)
const pairDebug = tupleDebug<readonly [number, string]>(intDebug, stringDebug)
assertEqual(pairShow.show([42, "ready"]), "(42, ready)")
assertEqual(pairDebug.debug([42, "ready"]), '(42, "ready")')
assertEqual(
  renderDebug(pairDebug, [42, "ready"], { layout: "multiline" }),
  '(\n  42,\n  "ready"\n)'
)

type Profile = Readonly<{
  alpha: number
  zeta?: string
}>
const profileShow = recordShow<Profile>(
  ["alpha", "zeta"],
  [false, true],
  intShow,
  stringShow
)
const profileDebug = recordDebug<Profile>(
  ["alpha", "zeta"],
  [false, true],
  intDebug,
  stringDebug
)
assertEqual(profileShow.show({ alpha: 1 }), "{ alpha: 1, zeta?: Nothing }")
assertEqual(
  profileShow.show({ zeta: "last", alpha: 1 }),
  "{ alpha: 1, zeta?: Just last }"
)
assertEqual(
  profileDebug.debug({ zeta: "last", alpha: 1 }),
  '{ alpha: 1, zeta?: Just "last" }'
)
assertEqual(
  renderDebug(
    profileDebug,
    { zeta: "last", alpha: 1 },
    {
      layout: "multiline",
    }
  ),
  '{\n  alpha: 1,\n  zeta?: Just "last"\n}'
)

type NestedRecord = Readonly<{
  pairs: ReadonlyArray<readonly [number, string]>
}>
const nestedRecordDebug = recordDebug<NestedRecord>(
  ["pairs"],
  [false],
  arrayDebug(pairDebug)
)
assertEqual(
  nestedRecordDebug.debug({ pairs: [[1, "one"]] }),
  '{ pairs: [(1, "one")] }'
)

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
assertEqual(
  consoleErrorDebug.debug(consoleError),
  'ConsoleError { message: "<redacted>" }'
)

const stdinCases: ReadonlyArray<readonly [StdinError, string]> = [
  [{ tag: "StdinUnavailable" }, "StdinUnavailable"],
  [{ tag: "StdinReadFailure" }, "StdinReadFailure"],
  [{ tag: "ConcurrentStdinRead" }, "ConcurrentStdinRead"],
  [
    { tag: "InvalidStdinUtf8", value: { offset: 12 } },
    "InvalidStdinUtf8 { offset: 12 }",
  ],
  [
    { tag: "StdinLineTooLong", value: { limitBytes: 1024 } },
    "StdinLineTooLong { limitBytes: 1024 }",
  ],
  [{ tag: "StdinPositionOverflow" }, "StdinPositionOverflow"],
]
for (const [error, expected] of stdinCases) {
  assertEqual(stdinErrorShow.show(error), expected)
  assertEqual(stdinErrorDebug.debug(error), expected)
}

const domCases: ReadonlyArray<readonly [DomError, string, string]> = [
  [
    { tag: "InvalidSelector", value: "[" },
    "InvalidSelector [",
    'InvalidSelector "["',
  ],
  [
    { tag: "DomTargetNotFound", value: "#missing" },
    "DomTargetNotFound #missing",
    'DomTargetNotFound "#missing"',
  ],
  [
    { tag: "DomTargetAlreadyMounted" },
    "DomTargetAlreadyMounted",
    "DomTargetAlreadyMounted",
  ],
  [
    {
      tag: "HydrationMismatch",
      value: {
        path: [0, 1],
        expected: "<span>client</span>",
        actual: "<span>server</span>",
      },
    },
    "HydrationMismatch { path: [0, 1], expected: <span>client</span>, actual: <span>server</span> }",
    'HydrationMismatch { path: [0, 1], expected: "<span>client</span>", actual: "<span>server</span>" }',
  ],
  [
    { tag: "DomEventQueueOverflow", value: 1024 },
    "DomEventQueueOverflow 1024",
    "DomEventQueueOverflow 1024",
  ],
  [{ tag: "DomTargetRemoved" }, "DomTargetRemoved", "DomTargetRemoved"],
  [
    { tag: "DomOperationFailed", value: "replace" },
    "DomOperationFailed replace",
    'DomOperationFailed "replace"',
  ],
]
for (const [error, shown, debugged] of domCases) {
  assertEqual(domErrorShow.show(error), shown)
  assertEqual(domErrorDebug.debug(error), debugged)
}

const dispatchFailure: DomRuntimeError<string> = {
  tag: "DispatchFailure",
  value: "denied",
}
assertEqual(
  domRuntimeErrorShow(stringShow).show(dispatchFailure),
  "DispatchFailure denied"
)
assertEqual(
  domRuntimeErrorDebug(stringDebug).debug(dispatchFailure),
  'DispatchFailure "denied"'
)
assertEqual(
  domRuntimeErrorShow(stringShow).show({
    tag: "DomFailure",
    value: { tag: "DomTargetRemoved" },
  }),
  "DomFailure DomTargetRemoved"
)

const htmlBuildError: HtmlBuildError = {
  tag: "ReservedAttributeName",
  value: "onclick",
}
assertEqual(
  htmlBuildErrorShow.show(htmlBuildError),
  "ReservedAttributeName onclick"
)
assertEqual(
  htmlBuildErrorDebug.debug(htmlBuildError),
  'ReservedAttributeName "onclick"'
)

function recursiveShowValue(depth: number): string {
  return depth === 0 ? "End" : `Link ${recursiveShow.show(depth - 1)}`
}
const recursiveShow: Show<number> = boundedShow(recursiveShowValue)
function recursiveDebugValue(depth: number): string {
  return depth === 0 ? "End" : `Link ${recursiveDebug.debug(depth - 1)}`
}
const recursiveDebug: Debug<number> = boundedDebug(recursiveDebugValue)
const deepShow = recursiveShow.show(displayDepthLimit * 2)
const deepDebug = recursiveDebug.debug(displayDepthLimit * 2)
if (!deepShow.endsWith("…") || !deepDebug.endsWith("…")) {
  throw new Error("recursive display did not stop at the shared depth limit")
}

process.stdout.write("show runtime probe passed\n")

import assert from "node:assert/strict"
import { singletonList } from "../src/list"
import {
  type Debug,
  displayDepthLimit,
  intDebug,
  intShow,
  listDebug,
  listShow,
  type Show,
  tupleDebug,
  tupleShow,
} from "../src/show"

const events: number[] = []
const selected: Show<number> = {
  show(value) {
    events.push(value)
    if (value === 2) throw new Error("second child defect")
    return `selected:${value}`
  },
}
const dictionary = tupleShow(selected, selected, selected)
assert.throws(() => dictionary.show([1, 2, 3]), /second child defect/)
assert.deepEqual(events, [1, 2])
events.length = 0
assert.equal(dictionary.show([3, 4, 5]), "(selected:3, selected:4, selected:5)")
assert.deepEqual(events, [3, 4, 5])

function expected(remaining: number, depth = 0): string {
  if (depth >= displayDepthLimit) return "…"
  if (remaining === 0) return "1"
  const child = expected(remaining - 1, depth + 1)
  return remaining % 2 === 0
    ? `(${child}, ${depth + 1 >= displayDepthLimit ? "…" : "0"})`
    : `\`[${child}]`
}

for (const debug of [false, true]) {
  let value: unknown = 1
  // The probe deliberately builds a heterogeneously nested runtime value;
  // generated program types are checked separately by the CLI fixture probe.
  let evidence: Show<any> | Debug<any> = debug ? intDebug : intShow
  for (let depth = 1; depth <= 130; depth++) {
    if (depth % 2 === 0) {
      value = [value, 0]
      evidence = debug
        ? tupleDebug(evidence as Debug<any>, intDebug)
        : tupleShow(evidence as Show<any>, intShow)
    } else {
      value = singletonList(value)
      evidence = debug
        ? listDebug(evidence as Debug<any>)
        : listShow(evidence as Show<any>)
    }
  }
  const output = debug
    ? (evidence as Debug<any>).debug(value)
    : (evidence as Show<any>).show(value)
  assert.equal(output, expected(130))
}
assert.equal(intShow.show(42), "42")
console.log("tuple selected methods, Defect order, depth and restoration agree")

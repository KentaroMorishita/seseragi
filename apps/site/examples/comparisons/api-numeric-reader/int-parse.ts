function readCount(text: string): number | undefined {
  const matched = /^[+-]?(0|[1-9][0-9]*)$/.exec(text)
  if (matched === null || matched[0] !== text) return undefined
  const value = Number(text)
  return Number.isSafeInteger(value) ? (value === 0 ? 0 : value) : undefined
}
function describeCount(text: string): string {
  const count = readCount(text)
  return count === undefined ? "invalid count" : `count: ${count}`
}
console.log(describeCount("+12"))
console.log(describeCount("-0"))
console.log(describeCount("012"))
console.log(describeCount("12px"))
console.log(describeCount("9007199254740991"))
console.log(describeCount("9007199254740992"))

export {}

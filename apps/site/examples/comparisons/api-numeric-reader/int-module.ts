function readCount(text: string): number | undefined {
  const matched = /^[+-]?(0|[1-9][0-9]*)$/.exec(text)
  if (matched === null || matched[0] !== text) return undefined
  const value = Number(text)
  return Number.isSafeInteger(value) ? (value === 0 ? 0 : value) : undefined
}
function addFive(text: string): string {
  const count = readCount(text)
  if (count === undefined) return "invalid count"
  const total = count + 5
  return Number.isSafeInteger(total) ? `total: ${total}` : "count too large"
}
console.log(addFive("12"))
console.log(addFive("12px"))
console.log(addFive("9007199254740991"))

export {}

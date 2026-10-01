function fromScalar(point: number): string | undefined {
  if (
    !Number.isInteger(point) ||
    point < 0 ||
    point > 0x10ffff ||
    (point >= 0xd800 && point <= 0xdfff)
  )
    return undefined
  return String.fromCodePoint(point)
}
function describe(point: number): string {
  const value = fromScalar(point)
  return value === undefined ? "not a scalar" : `found: ${value}`
}
for (const point of [128077, -1, 55296, 57343, 1114112]) {
  console.log(describe(point))
}
console.log(`zero accepted: ${fromScalar(0) === "\0"}`)
console.log(`last accepted: ${fromScalar(1114111) === "\u{10ffff}"}`)
console.log(
  `JS accepts surrogate: ${String.fromCodePoint(55296).charCodeAt(0)}`
)

export {}

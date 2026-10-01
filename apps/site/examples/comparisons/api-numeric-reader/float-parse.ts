function readFloat(text: string): number | undefined {
  if (["NaN", "Infinity", "-Infinity"].includes(text)) return Number(text)
  const decimal = /^[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?$/
  if (text.trim() !== text || !decimal.test(text)) return undefined
  const value = Number(text)
  return Number.isFinite(value) ? value : undefined
}

function describeMeasurement(text: string): string {
  const value = readFloat(text)
  if (value === undefined) return "invalid measurement"
  return Number.isFinite(value)
    ? `measurement: ${value}`
    : "non-finite measurement"
}

console.log(describeMeasurement("12.5"))
console.log(describeMeasurement("12.5cm"))
console.log(describeMeasurement(""))
console.log(describeMeasurement(" 12.5"))
console.log(describeMeasurement("Infinity"))
console.log(describeMeasurement("NaN"))
console.log(describeMeasurement("1e309"))

export {}

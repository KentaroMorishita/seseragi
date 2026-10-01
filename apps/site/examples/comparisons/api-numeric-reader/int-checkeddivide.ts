// Both inputs are already safe integers.
function calculate(dividend: number, divisor: number): string {
  if (divisor === 0) return "zero divisor"
  const result = Number(BigInt(dividend) / BigInt(divisor))
  return `result: ${result}`
}
console.log(calculate(10, 3))
console.log(calculate(-10, 3))
console.log(calculate(-9, 3))
console.log(calculate(10, 0))

export {}

// Both inputs are already safe integers.
function calculate(dividend: number, divisor: number): string {
  if (divisor === 0) return "zero divisor"
  const result = dividend % divisor
  return `result: ${result === 0 ? 0 : result}`
}
console.log(calculate(10, 3))
console.log(calculate(-10, 3))
console.log(calculate(-9, 3))
console.log(calculate(10, 0))

export {}

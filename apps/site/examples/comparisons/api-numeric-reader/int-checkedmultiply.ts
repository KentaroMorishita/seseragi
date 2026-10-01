// Both inputs are already safe integers.
function calculate(original: number, amount: number): string {
  const result = original * amount
  return Number.isSafeInteger(result) ? `result: ${result}` : "out of range"
}
console.log(calculate(12, 5))
console.log(calculate(9007199254740991, 2))

export {}

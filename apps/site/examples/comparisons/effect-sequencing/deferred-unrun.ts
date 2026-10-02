function divide(divisor: number): number {
  if (divisor === 0) throw new Error("cannot divide by zero")
  return Math.trunc(7 / divisor)
}
const _work = () => divide(0)
console.log("Constructed without executing")
export {}

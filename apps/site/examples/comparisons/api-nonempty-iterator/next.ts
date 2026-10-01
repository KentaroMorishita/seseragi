function* countdown(remaining: number) {
  while (remaining > 0) {
    yield remaining
    remaining -= 1
  }
}
function first(values: Iterator<number>): string {
  const result = values.next()
  return result.done ? "done" : String(result.value)
}
const values = countdown(3)
console.log(first(values))
console.log(first(values))
console.log(first(values))
console.log(first(countdown(0)))
export {}

function* countdown(remaining: number) {
  while (remaining > 0) {
    yield remaining
    remaining -= 1
  }
}
function* repeatValue(value: number) {
  while (true) {
    yield value
  }
}
console.log(JSON.stringify(Array.from(countdown(3))))
console.log(JSON.stringify(Array.from(countdown(0))))
const result = repeatValue(10).next()
console.log(result.done ? "done" : String(result.value))
export {}

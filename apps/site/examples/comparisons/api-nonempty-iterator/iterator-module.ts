function* countdown(remaining: number) {
  while (remaining > 0) {
    yield remaining
    remaining -= 1
  }
}
console.log(JSON.stringify(Array.from(countdown(3))))
console.log(JSON.stringify(Array.from(countdown(3))))
console.log(JSON.stringify(Array.from(countdown(0))))
export {}

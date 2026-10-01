// Keep a position as a value; next returns a new position for the rest.
type Countdown = { readonly remaining: number }
type Pull = { value: number; rest: Countdown }

function next(values: Countdown): Pull | undefined {
  return values.remaining > 0
    ? { value: values.remaining, rest: { remaining: values.remaining - 1 } }
    : undefined
}

function first(values: Countdown): string {
  const pulled = next(values)
  return pulled === undefined ? "done" : String(pulled.value)
}

function second(values: Countdown): string {
  const pulled = next(values)
  return pulled === undefined ? "done" : first(pulled.rest)
}

const values: Countdown = { remaining: 3 }
console.log(first(values))
console.log(first(values))
console.log(second(values))
console.log(first({ remaining: 0 }))

export {}

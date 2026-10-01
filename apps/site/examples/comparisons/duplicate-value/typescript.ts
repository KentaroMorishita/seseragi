function duplicate<A>(value: A): [A, A] {
  return [value, value]
}

const [left, right] = duplicate(21)
const [first, second] = duplicate<string>("ready")
console.log(`${left}, ${right}\n${first}, ${second}`)

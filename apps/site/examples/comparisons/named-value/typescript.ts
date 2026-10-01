type Named<A> = { label: string; value: A }

function readValue<A>(item: Named<A>): A {
  return item.value
}

const count: Named<number> = { label: "count", value: 42 }
const state: Named<string> = { label: "state", value: "ready" }
console.log(`${readValue(count)}\n${readValue(state)}`)

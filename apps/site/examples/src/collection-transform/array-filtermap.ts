const display = (value: unknown): void => console.log(JSON.stringify(value))
const inputs = ["2", "oops", "0", "3"]
const counts = inputs.flatMap((value) => {
  const count = Number(value)
  return Number.isInteger(count) ? [count] : []
})
display(counts)
display(inputs)

export {}

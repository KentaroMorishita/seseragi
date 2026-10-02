const display = (value: unknown): void => console.log(JSON.stringify(value))
const oldestFirst = ["Read", "Review", "Ship"]
display([...oldestFirst].reverse())
display(oldestFirst)

export {}

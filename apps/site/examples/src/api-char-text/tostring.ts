const symbol = "👍"
const value = symbol
const label = `Status: ${value}`
console.log(label)
console.log(`scalars: ${Array.from(value).length}`)
console.log(`UTF-8 bytes: ${new TextEncoder().encode(value).length}`)

export {}

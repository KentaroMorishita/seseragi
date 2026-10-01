const values = new Set([0])
console.log(JSON.stringify([...values]))
console.log(values.size)
console.log(values.has(0))
console.log(JSON.stringify([...new Set(values).add(0)]))
export {}

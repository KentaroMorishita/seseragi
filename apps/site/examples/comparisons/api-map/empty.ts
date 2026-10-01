const values = new Map<string, number>()
console.log(JSON.stringify([...values.entries()]))
console.log(values.size)
console.log(values.has("tea"))
export {}

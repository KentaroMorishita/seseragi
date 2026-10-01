const values = new Map([["tea", 2]])
console.log(JSON.stringify([...values.entries()]))
console.log(values.size)
console.log(JSON.stringify([...new Map([["empty", ""]]).entries()]))
export {}

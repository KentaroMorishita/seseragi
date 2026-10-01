const symbol = "👍"
console.log(`symbol: ${symbol.codePointAt(0)}`)
console.log(`letter: ${"A".codePointAt(0)}`)
console.log(`null: ${"\0".codePointAt(0)}`)
console.log(`first UTF-16 unit: ${symbol.charCodeAt(0)}`)
console.log(`UTF-16 units: ${symbol.length}`)

export {}

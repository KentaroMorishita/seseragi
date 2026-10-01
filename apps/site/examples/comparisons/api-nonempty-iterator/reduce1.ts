type NonEmpty<T> = readonly [T, ...T[]]
const amounts: NonEmpty<number> = [20, 3, 2]
const single: NonEmpty<number> = [20]
const subtract = (accumulated: number, value: number) => accumulated - value
console.log(amounts.reduce(subtract))
console.log(single.reduce(subtract))
console.log(JSON.stringify(amounts))
export {}

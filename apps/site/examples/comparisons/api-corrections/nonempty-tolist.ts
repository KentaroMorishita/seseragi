type NonEmpty<T> = readonly [T, ...T[]]
const single: NonEmpty<number> = [10]
const several: NonEmpty<number> = [10, 20, 30]
console.log(JSON.stringify(single.slice()))
console.log(JSON.stringify(several.slice()))
console.log(several[0])
export {}

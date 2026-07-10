export const add = (left: bigint, right: bigint) => left + right
export const addOne = (value: bigint) => add(value, 1n)
export const total = (unit: undefined) => addOne(41n)

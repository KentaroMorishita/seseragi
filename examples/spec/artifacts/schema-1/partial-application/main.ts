export const add = (left: bigint) => (right: bigint) => left + right
export const addTo = (value: bigint) => add(value)

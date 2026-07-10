export const isZero = (value: bigint) => value === 0n
export const sameBool = (left: boolean) => (right: boolean) => left === right
export const sameString = (left: string) => (right: string) => left !== right

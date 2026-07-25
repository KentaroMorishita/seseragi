export const duplicate = <A,>(value: A) => ({ "left": value, "right": value } as const)

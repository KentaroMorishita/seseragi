export const pair = (left: number) => (right: boolean) => [left, right] as const
export const sample: readonly [number, boolean] = [1, true] as const;

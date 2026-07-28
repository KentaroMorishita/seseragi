export const pair = (left: bigint) => (right: boolean) => [left, right] as const
export const sample: readonly [bigint, boolean] = [1n, true] as const;

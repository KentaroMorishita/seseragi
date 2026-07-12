export type Hand =
  | { readonly tag: "Rock" };
export const Rock: Hand = { tag: "Rock" } as const;
export const identity = <A,>(value: A) => value

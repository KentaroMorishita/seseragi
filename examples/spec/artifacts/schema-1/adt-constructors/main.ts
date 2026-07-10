export type Hand =
  | { readonly tag: "Rock" }
  | { readonly tag: "Paper" }
  | { readonly tag: "Scissors" };
export const Rock: Hand = { tag: "Rock" } as const;
export const Paper: Hand = { tag: "Paper" } as const;
export const Scissors: Hand = { tag: "Scissors" } as const;
export type Label =
  | { readonly tag: "Missing" }
  | { readonly tag: "Present"; readonly value: string };
export const Missing: Label = { tag: "Missing" } as const;
export const Present = (value: string): Label => ({ tag: "Present", value } as const);
export const opening: Hand = Rock;
export const wrap = (value: string) => Present(value)

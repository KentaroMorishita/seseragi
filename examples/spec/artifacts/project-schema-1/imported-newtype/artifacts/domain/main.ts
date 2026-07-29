export type UserId =
  | { readonly tag: "UserId"; readonly value: number };
export const UserId = (value: number): UserId => ({ tag: "UserId", value } as const);

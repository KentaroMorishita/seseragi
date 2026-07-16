export type UserId =
  | { readonly tag: "UserId"; readonly value: bigint };
export const UserId = (value: bigint): UserId => ({ tag: "UserId", value } as const);

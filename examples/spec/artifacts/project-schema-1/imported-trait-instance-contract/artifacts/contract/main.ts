export type Prefix =
  | { readonly tag: "Prefix"; readonly value: string };
export const Prefix = (value: string): Prefix => ({ tag: "Prefix", value } as const);

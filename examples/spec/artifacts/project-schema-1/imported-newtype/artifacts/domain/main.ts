import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export type UserId =
  | { readonly tag: "UserId"; readonly value: number };
export const UserId = (value: number): UserId => ({ tag: "UserId", value } as const);

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export type Prefix =
  | { readonly tag: "Prefix"; readonly value: string };
export const Prefix = (value: string): Prefix => ({ tag: "Prefix", value } as const);

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source167 = { file: "main.ssrg", line: 13, column: 3, uri: "seseragi://artifact/adt-constructors", start: 167, end: 180 };
$ssrg$assertUnicodeVersion("17.0.0")

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
export const wrap = (value: string) => $ssrg$atSource($ssrg$source167, () => Present(value), false)
export const opening: Hand = Rock;

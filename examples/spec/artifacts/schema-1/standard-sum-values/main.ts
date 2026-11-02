import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { Right as _ssrg_either_Right, Left as _ssrg_either_Left, Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
const $ssrg$source200 = { file: "main.ssrg", line: 8, column: 4, uri: "seseragi://artifact/standard-sum-values", start: 200, end: 210 };
const $ssrg$source212 = { file: "main.ssrg", line: 8, column: 16, uri: "seseragi://artifact/standard-sum-values", start: 212, end: 228 };
const $ssrg$source230 = { file: "main.ssrg", line: 8, column: 34, uri: "seseragi://artifact/standard-sum-values", start: 230, end: 239 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Hand =
  | { readonly tag: "Rock" };
export const Rock: Hand = { tag: "Rock" } as const;
export type HandInputError =
  | { readonly tag: "InvalidHand" };
export const InvalidHand: HandInputError = { tag: "InvalidHand" } as const;
export const standardSumValues = (unit: undefined) => [$ssrg$atSource($ssrg$source200, () => _ssrg_either_Right(Rock), false), $ssrg$atSource($ssrg$source212, () => _ssrg_either_Left(InvalidHand), false), $ssrg$atSource($ssrg$source230, () => _ssrg_maybe_Just(Rock), false), _ssrg_maybe_Nothing] as const

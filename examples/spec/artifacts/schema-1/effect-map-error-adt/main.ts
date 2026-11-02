import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { mapError as _ssrg_effect_mapError, fail as _ssrg_effect_fail } from "@seseragi/runtime/effect"
const $ssrg$source152 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/effect-map-error-adt", start: 152, end: 199 };
const $ssrg$source174 = { file: "main.ssrg", line: 8, column: 25, uri: "seseragi://artifact/effect-map-error-adt", start: 174, end: 198 };
const $ssrg$source180 = { file: "main.ssrg", line: 8, column: 31, uri: "seseragi://artifact/effect-map-error-adt", start: 180, end: 197 };
$ssrg$assertUnicodeVersion("17.0.0")

export type HandInputError =
  | { readonly tag: "UnknownHand"; readonly value: string };
export const UnknownHand = (value: string): HandInputError => ({ tag: "UnknownHand", value } as const);
export type AppError =
  | { readonly tag: "InvalidHand"; readonly value: HandInputError };
export const InvalidHand = (value: HandInputError): AppError => ({ tag: "InvalidHand", value } as const);
export const rejectUnknownHand = (input: string) => $ssrg$atSource($ssrg$source152, () => _ssrg_effect_mapError(InvalidHand, $ssrg$atSource($ssrg$source174, () => _ssrg_effect_fail($ssrg$atSource($ssrg$source180, () => UnknownHand(input), false)), true)), true)

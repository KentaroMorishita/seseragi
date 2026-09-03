import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { mapError as _ssrg_effect_mapError, fail as _ssrg_effect_fail } from "@seseragi/runtime/effect"
$ssrg$assertUnicodeVersion("17.0.0")

export type HandInputError =
  | { readonly tag: "UnknownHand"; readonly value: string };
export const UnknownHand = (value: string): HandInputError => ({ tag: "UnknownHand", value } as const);
export type AppError =
  | { readonly tag: "InvalidHand"; readonly value: HandInputError };
export const InvalidHand = (value: HandInputError): AppError => ({ tag: "InvalidHand", value } as const);
export const rejectUnknownHand = (input: string) => _ssrg_effect_mapError(InvalidHand, _ssrg_effect_fail(UnknownHand(input)))

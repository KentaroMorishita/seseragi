import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { Right as _ssrg_either_Right, Left as _ssrg_either_Left } from "@seseragi/runtime/sum"
const $ssrg$source194 = { file: "main.ssrg", line: 11, column: 15, uri: "seseragi://artifact/parse-hand-either", start: 194, end: 204 };
const $ssrg$source220 = { file: "main.ssrg", line: 12, column: 16, uri: "seseragi://artifact/parse-hand-either", start: 220, end: 231 };
const $ssrg$source250 = { file: "main.ssrg", line: 13, column: 19, uri: "seseragi://artifact/parse-hand-either", start: 250, end: 264 };
const $ssrg$source278 = { file: "main.ssrg", line: 14, column: 14, uri: "seseragi://artifact/parse-hand-either", start: 278, end: 302 };
const $ssrg$source284 = { file: "main.ssrg", line: 14, column: 20, uri: "seseragi://artifact/parse-hand-either", start: 284, end: 301 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Hand =
  | { readonly tag: "Rock" }
  | { readonly tag: "Paper" }
  | { readonly tag: "Scissors" };
export const Rock: Hand = { tag: "Rock" } as const;
export const Paper: Hand = { tag: "Paper" } as const;
export const Scissors: Hand = { tag: "Scissors" } as const;
export type HandInputError =
  | { readonly tag: "UnknownHand"; readonly value: string };
export const UnknownHand = (value: string): HandInputError => ({ tag: "UnknownHand", value } as const);
export const parseHand = (input: string) => (($ssrg_match: string): { readonly tag: "Left"; readonly value: HandInputError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg_match === "rock" ? $ssrg$atSource($ssrg$source194, () => _ssrg_either_Right(Rock), false) : $ssrg_match === "paper" ? $ssrg$atSource($ssrg$source220, () => _ssrg_either_Right(Paper), false) : $ssrg_match === "scissors" ? $ssrg$atSource($ssrg$source250, () => _ssrg_either_Right(Scissors), false) : ((other: string): { readonly tag: "Left"; readonly value: HandInputError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg$atSource($ssrg$source278, () => _ssrg_either_Left($ssrg$atSource($ssrg$source284, () => UnknownHand(other), false)), false))($ssrg_match))(input)

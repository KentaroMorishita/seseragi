import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { boundedShow as _ssrg_show_boundedShow, stringShow as _ssrg_show_stringShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { Right as _ssrg_either_Right, Left as _ssrg_either_Left } from "@seseragi/runtime/sum"
import { fromEither as _ssrg_effect_fromEither } from "@seseragi/runtime/effect"
const $ssrg$source204 = { file: "main.ssrg", line: 11, column: 15, uri: "seseragi://artifact/effect-parse-hand", start: 204, end: 214 };
const $ssrg$source230 = { file: "main.ssrg", line: 12, column: 16, uri: "seseragi://artifact/effect-parse-hand", start: 230, end: 241 };
const $ssrg$source260 = { file: "main.ssrg", line: 13, column: 19, uri: "seseragi://artifact/effect-parse-hand", start: 260, end: 274 };
const $ssrg$source288 = { file: "main.ssrg", line: 14, column: 14, uri: "seseragi://artifact/effect-parse-hand", start: 288, end: 312 };
const $ssrg$source294 = { file: "main.ssrg", line: 14, column: 20, uri: "seseragi://artifact/effect-parse-hand", start: 294, end: 311 };
const $ssrg$source356 = { file: "main.ssrg", line: 18, column: 3, uri: "seseragi://artifact/effect-parse-hand", start: 356, end: 384 };
const $ssrg$source368 = { file: "main.ssrg", line: 18, column: 15, uri: "seseragi://artifact/effect-parse-hand", start: 368, end: 383 };
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
export const __ssrg$instance$Show$0: _ssrg_show_Show<HandInputError> = _ssrg_show_boundedShow((value: HandInputError): string => { switch (value.tag) { case "UnknownHand": return "UnknownHand" + " " + _ssrg_show_stringShow.show(value.value); } });
const parseHand = (input: string) => (($ssrg_match: string): { readonly tag: "Left"; readonly value: HandInputError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg_match === "rock" ? $ssrg$atSource($ssrg$source204, () => _ssrg_either_Right(Rock), false) : $ssrg_match === "paper" ? $ssrg$atSource($ssrg$source230, () => _ssrg_either_Right(Paper), false) : $ssrg_match === "scissors" ? $ssrg$atSource($ssrg$source260, () => _ssrg_either_Right(Scissors), false) : ((other: string): { readonly tag: "Left"; readonly value: HandInputError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg$atSource($ssrg$source288, () => _ssrg_either_Left($ssrg$atSource($ssrg$source294, () => UnknownHand(other), false)), false))($ssrg_match))(input)
export const parse = (input: string) => $ssrg$atSource($ssrg$source356, () => _ssrg_effect_fromEither($ssrg$atSource($ssrg$source368, () => parseHand(input), false)), true)

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source186 = { file: "main.ssrg", line: 12, column: 15, uri: "seseragi://artifact/parse-hand-literals", start: 186, end: 197 };
const $ssrg$source213 = { file: "main.ssrg", line: 13, column: 16, uri: "seseragi://artifact/parse-hand-literals", start: 213, end: 225 };
const $ssrg$source244 = { file: "main.ssrg", line: 14, column: 19, uri: "seseragi://artifact/parse-hand-literals", start: 244, end: 259 };
const $ssrg$source273 = { file: "main.ssrg", line: 15, column: 14, uri: "seseragi://artifact/parse-hand-literals", start: 273, end: 290 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Hand =
  | { readonly tag: "Rock" }
  | { readonly tag: "Paper" }
  | { readonly tag: "Scissors" };
export const Rock: Hand = { tag: "Rock" } as const;
export const Paper: Hand = { tag: "Paper" } as const;
export const Scissors: Hand = { tag: "Scissors" } as const;
export type HandParse =
  | { readonly tag: "Parsed"; readonly value: Hand }
  | { readonly tag: "UnknownHand"; readonly value: string };
export const Parsed = (value: Hand): HandParse => ({ tag: "Parsed", value } as const);
export const UnknownHand = (value: string): HandParse => ({ tag: "UnknownHand", value } as const);
export const parseHand = (input: string) => (($ssrg_match: string): HandParse => $ssrg_match === "rock" ? $ssrg$atSource($ssrg$source186, () => Parsed(Rock), false) : $ssrg_match === "paper" ? $ssrg$atSource($ssrg$source213, () => Parsed(Paper), false) : $ssrg_match === "scissors" ? $ssrg$atSource($ssrg$source244, () => Parsed(Scissors), false) : ((other: string): HandParse => $ssrg$atSource($ssrg$source273, () => UnknownHand(other), false))($ssrg_match))(input)

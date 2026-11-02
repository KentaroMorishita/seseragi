import { type Hand, Rock, Paper, Scissors } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { boundedShow as _ssrg_show_boundedShow, stdinErrorShow as _ssrg_show_stdinErrorShow, stringShow as _ssrg_show_stringShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { Right as _ssrg_either_Right, Left as _ssrg_either_Left } from "@seseragi/runtime/sum"
import { flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError, fromEither as _ssrg_effect_fromEither } from "@seseragi/runtime/effect"
import { readLine as _ssrg_stdin_readLine, type StdinError as StdinError } from "@seseragi/runtime/stdin"
const $ssrg$source246 = { file: "project/rock-paper-scissors-cli-split/src/input.ssrg", line: 10, column: 15, uri: "seseragi://fixture/rps-cli-split::input", start: 246, end: 256 };
const $ssrg$source272 = { file: "project/rock-paper-scissors-cli-split/src/input.ssrg", line: 11, column: 16, uri: "seseragi://fixture/rps-cli-split::input", start: 272, end: 283 };
const $ssrg$source302 = { file: "project/rock-paper-scissors-cli-split/src/input.ssrg", line: 12, column: 19, uri: "seseragi://fixture/rps-cli-split::input", start: 302, end: 316 };
const $ssrg$source330 = { file: "project/rock-paper-scissors-cli-split/src/input.ssrg", line: 13, column: 14, uri: "seseragi://fixture/rps-cli-split::input", start: 330, end: 354 };
const $ssrg$source336 = { file: "project/rock-paper-scissors-cli-split/src/input.ssrg", line: 13, column: 20, uri: "seseragi://fixture/rps-cli-split::input", start: 336, end: 353 };
const $ssrg$source456 = { file: "project/rock-paper-scissors-cli-split/src/input.ssrg", line: 18, column: 16, uri: "seseragi://fixture/rps-cli-split::input", start: 456, end: 471 };
const $ssrg$source489 = { file: "project/rock-paper-scissors-cli-split/src/input.ssrg", line: 19, column: 18, uri: "seseragi://fixture/rps-cli-split::input", start: 489, end: 503 };
const $ssrg$source554 = { file: "project/rock-paper-scissors-cli-split/src/input.ssrg", line: 24, column: 14, uri: "seseragi://fixture/rps-cli-split::input", start: 554, end: 589 };
const $ssrg$source577 = { file: "project/rock-paper-scissors-cli-split/src/input.ssrg", line: 24, column: 37, uri: "seseragi://fixture/rps-cli-split::input", start: 577, end: 588 };
const $ssrg$source594 = { file: "project/rock-paper-scissors-cli-split/src/input.ssrg", line: 25, column: 5, uri: "seseragi://fixture/rps-cli-split::input", start: 594, end: 623 };
const $ssrg$source606 = { file: "project/rock-paper-scissors-cli-split/src/input.ssrg", line: 25, column: 17, uri: "seseragi://fixture/rps-cli-split::input", start: 606, end: 622 };
$ssrg$assertUnicodeVersion("17.0.0")

export type InputError =
  | { readonly tag: "StdinFailure"; readonly value: StdinError }
  | { readonly tag: "EndOfInput" }
  | { readonly tag: "UnknownHand"; readonly value: string };
export const StdinFailure = (value: StdinError): InputError => ({ tag: "StdinFailure", value } as const);
export const EndOfInput: InputError = { tag: "EndOfInput" } as const;
export const UnknownHand = (value: string): InputError => ({ tag: "UnknownHand", value } as const);
export const __ssrg$instance$Show$0: _ssrg_show_Show<InputError> = _ssrg_show_boundedShow((value: InputError): string => { switch (value.tag) { case "StdinFailure": return "StdinFailure" + " " + _ssrg_show_stdinErrorShow.show(value.value); case "EndOfInput": return "EndOfInput"; case "UnknownHand": return "UnknownHand" + " " + _ssrg_show_stringShow.show(value.value); } });
const parseHand = (input: string) => (($ssrg_match: string): { readonly tag: "Left"; readonly value: InputError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg_match === "rock" ? $ssrg$atSource($ssrg$source246, () => _ssrg_either_Right(Rock), false) : $ssrg_match === "paper" ? $ssrg$atSource($ssrg$source272, () => _ssrg_either_Right(Paper), false) : $ssrg_match === "scissors" ? $ssrg$atSource($ssrg$source302, () => _ssrg_either_Right(Scissors), false) : ((other: string): { readonly tag: "Left"; readonly value: InputError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg$atSource($ssrg$source330, () => _ssrg_either_Left($ssrg$atSource($ssrg$source336, () => UnknownHand(other), false)), false))($ssrg_match))(input)
const parseInput = (input: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }): { readonly tag: "Left"; readonly value: InputError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg_match.tag === "Nothing" ? $ssrg$atSource($ssrg$source456, () => _ssrg_either_Left(EndOfInput), false) : $ssrg_match.tag === "Just" ? ((line: string): { readonly tag: "Left"; readonly value: InputError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg$atSource($ssrg$source489, () => parseHand(line), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(input)
export const readHand = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source554, () => _ssrg_effect_mapError(StdinFailure, $ssrg$atSource($ssrg$source577, () => _ssrg_stdin_readLine(), true)), true), (input: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => $ssrg$atSource($ssrg$source594, () => _ssrg_effect_fromEither($ssrg$atSource($ssrg$source606, () => parseInput(input), false)), true))

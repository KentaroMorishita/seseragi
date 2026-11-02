import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { boundedShow as _ssrg_show_boundedShow, stdinErrorShow as _ssrg_show_stdinErrorShow, stringShow as _ssrg_show_stringShow, consoleErrorShow as _ssrg_show_consoleErrorShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { Right as _ssrg_either_Right, Left as _ssrg_either_Left } from "@seseragi/runtime/sum"
import { flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError, fromEither as _ssrg_effect_fromEither } from "@seseragi/runtime/effect"
import { readLine as _ssrg_stdin_readLine, type StdinError as StdinError } from "@seseragi/runtime/stdin"
import { println as _ssrg_console_println, type ConsoleError as ConsoleError } from "@seseragi/runtime/console"
const $ssrg$source324 = { file: "main.ssrg", line: 18, column: 13, uri: "seseragi://artifact/rock-paper-scissors-cli", start: 324, end: 334 };
const $ssrg$source348 = { file: "main.ssrg", line: 19, column: 14, uri: "seseragi://artifact/rock-paper-scissors-cli", start: 348, end: 359 };
const $ssrg$source376 = { file: "main.ssrg", line: 20, column: 17, uri: "seseragi://artifact/rock-paper-scissors-cli", start: 376, end: 390 };
const $ssrg$source402 = { file: "main.ssrg", line: 21, column: 12, uri: "seseragi://artifact/rock-paper-scissors-cli", start: 402, end: 426 };
const $ssrg$source408 = { file: "main.ssrg", line: 21, column: 18, uri: "seseragi://artifact/rock-paper-scissors-cli", start: 408, end: 425 };
const $ssrg$source520 = { file: "main.ssrg", line: 25, column: 14, uri: "seseragi://artifact/rock-paper-scissors-cli", start: 520, end: 535 };
const $ssrg$source551 = { file: "main.ssrg", line: 26, column: 16, uri: "seseragi://artifact/rock-paper-scissors-cli", start: 551, end: 565 };
const $ssrg$source1038 = { file: "main.ssrg", line: 46, column: 17, uri: "seseragi://artifact/rock-paper-scissors-cli", start: 1038, end: 1049 };
const $ssrg$source1086 = { file: "main.ssrg", line: 47, column: 12, uri: "seseragi://artifact/rock-paper-scissors-cli", start: 1086, end: 1110 };
const $ssrg$source1142 = { file: "main.ssrg", line: 48, column: 18, uri: "seseragi://artifact/rock-paper-scissors-cli", start: 1142, end: 1153 };
const $ssrg$source1191 = { file: "main.ssrg", line: 49, column: 13, uri: "seseragi://artifact/rock-paper-scissors-cli", start: 1191, end: 1216 };
const $ssrg$source1247 = { file: "main.ssrg", line: 50, column: 17, uri: "seseragi://artifact/rock-paper-scissors-cli", start: 1247, end: 1269 };
const $ssrg$source1272 = { file: "main.ssrg", line: 51, column: 3, uri: "seseragi://artifact/rock-paper-scissors-cli", start: 1272, end: 1296 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Hand =
  | { readonly tag: "Rock" }
  | { readonly tag: "Paper" }
  | { readonly tag: "Scissors" };
export const Rock: Hand = { tag: "Rock" } as const;
export const Paper: Hand = { tag: "Paper" } as const;
export const Scissors: Hand = { tag: "Scissors" } as const;
export type Outcome =
  | { readonly tag: "Player1Wins" }
  | { readonly tag: "Player2Wins" }
  | { readonly tag: "Draw" };
export const Player1Wins: Outcome = { tag: "Player1Wins" } as const;
export const Player2Wins: Outcome = { tag: "Player2Wins" } as const;
export const Draw: Outcome = { tag: "Draw" } as const;
export type AppError =
  | { readonly tag: "StdinFailure"; readonly value: StdinError }
  | { readonly tag: "EndOfInput" }
  | { readonly tag: "UnknownHand"; readonly value: string }
  | { readonly tag: "ConsoleFailure"; readonly value: ConsoleError };
export const StdinFailure = (value: StdinError): AppError => ({ tag: "StdinFailure", value } as const);
export const EndOfInput: AppError = { tag: "EndOfInput" } as const;
export const UnknownHand = (value: string): AppError => ({ tag: "UnknownHand", value } as const);
export const ConsoleFailure = (value: ConsoleError): AppError => ({ tag: "ConsoleFailure", value } as const);
export const __ssrg$instance$Show$0: _ssrg_show_Show<AppError> = _ssrg_show_boundedShow((value: AppError): string => { switch (value.tag) { case "StdinFailure": return "StdinFailure" + " " + _ssrg_show_stdinErrorShow.show(value.value); case "EndOfInput": return "EndOfInput"; case "UnknownHand": return "UnknownHand" + " " + _ssrg_show_stringShow.show(value.value); case "ConsoleFailure": return "ConsoleFailure" + " " + _ssrg_show_consoleErrorShow.show(value.value); } });
const parseHand = (input: string) => (($ssrg_match: string): { readonly tag: "Left"; readonly value: AppError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg_match === "rock" ? $ssrg$atSource($ssrg$source324, () => _ssrg_either_Right(Rock), false) : $ssrg_match === "paper" ? $ssrg$atSource($ssrg$source348, () => _ssrg_either_Right(Paper), false) : $ssrg_match === "scissors" ? $ssrg$atSource($ssrg$source376, () => _ssrg_either_Right(Scissors), false) : ((other: string): { readonly tag: "Left"; readonly value: AppError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg$atSource($ssrg$source402, () => _ssrg_either_Left($ssrg$atSource($ssrg$source408, () => UnknownHand(other), false)), false))($ssrg_match))(input)
const parseInput = (input: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }): { readonly tag: "Left"; readonly value: AppError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg_match.tag === "Nothing" ? $ssrg$atSource($ssrg$source520, () => _ssrg_either_Left(EndOfInput), false) : $ssrg_match.tag === "Just" ? ((line: string): { readonly tag: "Left"; readonly value: AppError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg$atSource($ssrg$source551, () => parseHand(line), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(input)
const decide = (first: Hand) => (second: Hand) => (($ssrg_match: readonly [Hand, Hand]): Outcome => $ssrg_match[0].tag === "Rock" && $ssrg_match[1].tag === "Rock" ? Draw : $ssrg_match[0].tag === "Paper" && $ssrg_match[1].tag === "Paper" ? Draw : $ssrg_match[0].tag === "Scissors" && $ssrg_match[1].tag === "Scissors" ? Draw : $ssrg_match[0].tag === "Rock" && $ssrg_match[1].tag === "Scissors" ? Player1Wins : $ssrg_match[0].tag === "Paper" && $ssrg_match[1].tag === "Rock" ? Player1Wins : $ssrg_match[0].tag === "Scissors" && $ssrg_match[1].tag === "Paper" ? Player1Wins : Player2Wins)([first, second] as const)
const renderOutcome = (outcome: Outcome) => (($ssrg_match: Outcome): string => $ssrg_match.tag === "Player1Wins" ? "Player 1 wins!" : $ssrg_match.tag === "Player2Wins" ? "Player 2 wins!" : "Draw!")(outcome)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1038, () => _ssrg_effect_mapError(StdinFailure, $ssrg$atSource($ssrg$source1038, () => _ssrg_stdin_readLine(), true)), true), (firstInput: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1086, () => _ssrg_effect_fromEither($ssrg$atSource($ssrg$source1086, () => parseInput(firstInput), false)), true), (first: Hand) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1142, () => _ssrg_effect_mapError(StdinFailure, $ssrg$atSource($ssrg$source1142, () => _ssrg_stdin_readLine(), true)), true), (secondInput: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1191, () => _ssrg_effect_fromEither($ssrg$atSource($ssrg$source1191, () => parseInput(secondInput), false)), true), (second: Hand) => (() => { const outcome: Outcome = $ssrg$atSource($ssrg$source1247, () => decide(first)(second), false); return $ssrg$atSource($ssrg$source1272, () => _ssrg_effect_mapError(ConsoleFailure, $ssrg$atSource($ssrg$source1272, () => _ssrg_console_println($ssrg$atSource($ssrg$source1272, () => renderOutcome(outcome), false)), true)), true); })()))))

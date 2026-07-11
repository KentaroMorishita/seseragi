import { Right as _ssrg_either_Right, Left as _ssrg_either_Left } from "@seseragi/runtime/sum"
import { flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError, fromEither as _ssrg_effect_fromEither } from "@seseragi/runtime/effect"
import { readLine as _ssrg_stdin_readLine, type StdinError as StdinError } from "@seseragi/runtime/stdin"
import { println as _ssrg_console_println, type ConsoleError as ConsoleError } from "@seseragi/runtime/console"

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
const parseHand = (input: string) => (($ssrg_match: string): { readonly tag: "Left"; readonly value: AppError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg_match === "rock" ? _ssrg_either_Right(Rock) : $ssrg_match === "paper" ? _ssrg_either_Right(Paper) : $ssrg_match === "scissors" ? _ssrg_either_Right(Scissors) : ((other: string): { readonly tag: "Left"; readonly value: AppError } | { readonly tag: "Right"; readonly value: Hand } => _ssrg_either_Left(UnknownHand(other)))($ssrg_match))(input)
const parseInput = (input: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }): { readonly tag: "Left"; readonly value: AppError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg_match.tag === "Nothing" ? _ssrg_either_Left(EndOfInput) : $ssrg_match.tag === "Just" ? ((line: string): { readonly tag: "Left"; readonly value: AppError } | { readonly tag: "Right"; readonly value: Hand } => parseHand(line))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(input)
const decide = (first: Hand) => (second: Hand) => (($ssrg_match: readonly [Hand, Hand]): Outcome => $ssrg_match[0].tag === "Rock" && $ssrg_match[1].tag === "Rock" ? Draw : $ssrg_match[0].tag === "Paper" && $ssrg_match[1].tag === "Paper" ? Draw : $ssrg_match[0].tag === "Scissors" && $ssrg_match[1].tag === "Scissors" ? Draw : $ssrg_match[0].tag === "Rock" && $ssrg_match[1].tag === "Scissors" ? Player1Wins : $ssrg_match[0].tag === "Paper" && $ssrg_match[1].tag === "Rock" ? Player1Wins : $ssrg_match[0].tag === "Scissors" && $ssrg_match[1].tag === "Paper" ? Player1Wins : Player2Wins)([first, second] as const)
const renderOutcome = (outcome: Outcome) => (($ssrg_match: Outcome): string => $ssrg_match.tag === "Player1Wins" ? "Player 1 wins!" : $ssrg_match.tag === "Player2Wins" ? "Player 2 wins!" : "Draw!")(outcome)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_effect_mapError(StdinFailure, _ssrg_stdin_readLine()), (firstInput: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => _ssrg_effect_flatMap(_ssrg_effect_fromEither(parseInput(firstInput)), (first: Hand) => _ssrg_effect_flatMap(_ssrg_effect_mapError(StdinFailure, _ssrg_stdin_readLine()), (secondInput: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => _ssrg_effect_flatMap(_ssrg_effect_fromEither(parseInput(secondInput)), (second: Hand) => (() => { const outcome: Outcome = decide(first)(second); return _ssrg_effect_mapError(ConsoleFailure, _ssrg_console_println(renderOutcome(outcome))); })()))))

import { type Hand, Rock, Paper, Scissors } from "./domain.js"
import { stdinErrorShow as _ssrg_show_stdinErrorShow, stringShow as _ssrg_show_stringShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { Right as _ssrg_either_Right, Left as _ssrg_either_Left } from "@seseragi/runtime/sum"
import { flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError, fromEither as _ssrg_effect_fromEither } from "@seseragi/runtime/effect"
import { readLine as _ssrg_stdin_readLine, type StdinError as StdinError } from "@seseragi/runtime/stdin"

export type InputError =
  | { readonly tag: "StdinFailure"; readonly value: StdinError }
  | { readonly tag: "EndOfInput" }
  | { readonly tag: "UnknownHand"; readonly value: string };
export const StdinFailure = (value: StdinError): InputError => ({ tag: "StdinFailure", value } as const);
export const EndOfInput: InputError = { tag: "EndOfInput" } as const;
export const UnknownHand = (value: string): InputError => ({ tag: "UnknownHand", value } as const);
export const __ssrg$instance$Show$0: _ssrg_show_Show<InputError> = { show: (value: InputError): string => { switch (value.tag) { case "StdinFailure": return "StdinFailure" + " " + _ssrg_show_stdinErrorShow.show(value.value); case "EndOfInput": return "EndOfInput"; case "UnknownHand": return "UnknownHand" + " " + _ssrg_show_stringShow.show(value.value); } } };
const parseHand = (input: string) => (($ssrg_match: string): { readonly tag: "Left"; readonly value: InputError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg_match === "rock" ? _ssrg_either_Right(Rock) : $ssrg_match === "paper" ? _ssrg_either_Right(Paper) : $ssrg_match === "scissors" ? _ssrg_either_Right(Scissors) : ((other: string): { readonly tag: "Left"; readonly value: InputError } | { readonly tag: "Right"; readonly value: Hand } => _ssrg_either_Left(UnknownHand(other)))($ssrg_match))(input)
const parseInput = (input: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }): { readonly tag: "Left"; readonly value: InputError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg_match.tag === "Nothing" ? _ssrg_either_Left(EndOfInput) : $ssrg_match.tag === "Just" ? ((line: string): { readonly tag: "Left"; readonly value: InputError } | { readonly tag: "Right"; readonly value: Hand } => parseHand(line))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(input)
export const readHand = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_effect_mapError(StdinFailure, _ssrg_stdin_readLine()), (input: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => _ssrg_effect_fromEither(parseInput(input)))

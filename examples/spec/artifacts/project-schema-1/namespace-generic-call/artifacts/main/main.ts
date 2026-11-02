import { type Hand as domain$member$Hand, Rock as domain$member$Rock, Paper as domain$member$Paper, Scissors as domain$member$Scissors, identity as domain$member$identity } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source431 = { file: "project/namespace-generic-call/src/main.ssrg", line: 20, column: 3, uri: "seseragi://fixture/namespace-call::main", start: 431, end: 488 };
const $ssrg$source448 = { file: "project/namespace-generic-call/src/main.ssrg", line: 20, column: 20, uri: "seseragi://fixture/namespace-call::main", start: 448, end: 487 };
const $ssrg$source456 = { file: "project/namespace-generic-call/src/main.ssrg", line: 20, column: 28, uri: "seseragi://fixture/namespace-call::main", start: 456, end: 486 };
const $ssrg$source473 = { file: "project/namespace-generic-call/src/main.ssrg", line: 20, column: 45, uri: "seseragi://fixture/namespace-call::main", start: 473, end: 485 };
$ssrg$assertUnicodeVersion("17.0.0")

export const keep = (value: domain$member$Hand) => value
export const choose = (value: string) => (($ssrg_match: string): domain$member$Hand => $ssrg_match === "paper" ? domain$member$Paper : $ssrg_match === "scissors" ? domain$member$Scissors : domain$member$Rock)(value)
export const render = (hand: domain$member$Hand) => (($ssrg_match: domain$member$Hand): string => $ssrg_match.tag === "Rock" ? "rock" : $ssrg_match.tag === "Paper" ? "paper" : "scissors")(hand)
export const run = (value: string) => $ssrg$atSource($ssrg$source431, () => domain$member$identity<string>($ssrg$atSource($ssrg$source448, () => render($ssrg$atSource($ssrg$source456, () => domain$member$identity<domain$member$Hand>($ssrg$atSource($ssrg$source473, () => choose(value), false)), false)), false)), false)

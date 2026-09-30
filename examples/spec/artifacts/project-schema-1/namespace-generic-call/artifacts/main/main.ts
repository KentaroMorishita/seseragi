import { type Hand as domain$member$Hand, Rock as domain$member$Rock, Paper as domain$member$Paper, Scissors as domain$member$Scissors, identity as domain$member$identity } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export const keep = (value: domain$member$Hand) => value
export const choose = (value: string) => (($ssrg_match: string): domain$member$Hand => $ssrg_match === "paper" ? domain$member$Paper : $ssrg_match === "scissors" ? domain$member$Scissors : domain$member$Rock)(value)
export const render = (hand: domain$member$Hand) => (($ssrg_match: domain$member$Hand): string => $ssrg_match.tag === "Rock" ? "rock" : $ssrg_match.tag === "Paper" ? "paper" : "scissors")(hand)
export const run = (value: string) => domain$member$identity(render(domain$member$identity(choose(value))))

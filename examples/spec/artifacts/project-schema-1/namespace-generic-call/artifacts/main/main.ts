import { type Hand as domain_Hand, Rock as domain_Rock, Paper as domain_Paper, Scissors as domain_Scissors, identity as domain_identity } from "./domain.js"

export const keep = (value: domain_Hand) => value
export const choose = (value: string) => (($ssrg_match: string): domain_Hand => $ssrg_match === "paper" ? domain_Paper : $ssrg_match === "scissors" ? domain_Scissors : domain_Rock)(value)
export const render = (hand: domain_Hand) => (($ssrg_match: domain_Hand): string => $ssrg_match.tag === "Rock" ? "rock" : $ssrg_match.tag === "Paper" ? "paper" : "scissors")(hand)
export const run = (value: string) => domain_identity(render(domain_identity(choose(value))))

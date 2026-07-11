import { Right as _ssrg_either_Right, Left as _ssrg_either_Left } from "@seseragi/runtime/sum"

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
export const parseHand = (input: string) => (($ssrg_match: string): { readonly tag: "Left"; readonly value: HandInputError } | { readonly tag: "Right"; readonly value: Hand } => $ssrg_match === "rock" ? _ssrg_either_Right(Rock) : $ssrg_match === "paper" ? _ssrg_either_Right(Paper) : $ssrg_match === "scissors" ? _ssrg_either_Right(Scissors) : ((other: string): { readonly tag: "Left"; readonly value: HandInputError } | { readonly tag: "Right"; readonly value: Hand } => _ssrg_either_Left(UnknownHand(other)))($ssrg_match))(input)

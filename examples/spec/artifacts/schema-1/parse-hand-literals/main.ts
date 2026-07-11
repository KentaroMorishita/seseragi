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
export const parseHand = (input: string) => (($ssrg_match: string): HandParse => $ssrg_match === "rock" ? Parsed(Rock) : $ssrg_match === "paper" ? Parsed(Paper) : $ssrg_match === "scissors" ? Parsed(Scissors) : ((other: string): HandParse => UnknownHand(other))($ssrg_match))(input)

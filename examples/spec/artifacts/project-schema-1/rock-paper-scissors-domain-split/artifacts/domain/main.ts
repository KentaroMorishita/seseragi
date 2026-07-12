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
export const identity = <A,>(value: A) => value
export const decide = (first: Hand) => (second: Hand) => (($ssrg_match: readonly [Hand, Hand]): Outcome => $ssrg_match[0].tag === "Rock" && $ssrg_match[1].tag === "Rock" ? Draw : $ssrg_match[0].tag === "Paper" && $ssrg_match[1].tag === "Paper" ? Draw : $ssrg_match[0].tag === "Scissors" && $ssrg_match[1].tag === "Scissors" ? Draw : $ssrg_match[0].tag === "Rock" && $ssrg_match[1].tag === "Scissors" ? Player1Wins : $ssrg_match[0].tag === "Paper" && $ssrg_match[1].tag === "Rock" ? Player1Wins : $ssrg_match[0].tag === "Scissors" && $ssrg_match[1].tag === "Paper" ? Player1Wins : Player2Wins)([first, second] as const)
export const renderOutcome = (outcome: Outcome) => (($ssrg_match: Outcome): string => $ssrg_match.tag === "Player1Wins" ? "Player 1 wins!" : $ssrg_match.tag === "Player2Wins" ? "Player 2 wins!" : "Draw!")(outcome)

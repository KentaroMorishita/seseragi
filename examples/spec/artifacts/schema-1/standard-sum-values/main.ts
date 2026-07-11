import { Right as _ssrg_either_Right, Left as _ssrg_either_Left, Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"

export type Hand =
  | { readonly tag: "Rock" };
export const Rock: Hand = { tag: "Rock" } as const;
export type HandInputError =
  | { readonly tag: "InvalidHand" };
export const InvalidHand: HandInputError = { tag: "InvalidHand" } as const;
export const standardSumValues = (unit: undefined) => [_ssrg_either_Right(Rock), _ssrg_either_Left(InvalidHand), _ssrg_maybe_Just(Rock), _ssrg_maybe_Nothing] as const

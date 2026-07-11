import { fail as _ssrg_effect_fail } from "@seseragi/runtime/effect"

export type AppError =
  | { readonly tag: "Invalid" };
export const Invalid: AppError = { tag: "Invalid" } as const;
export const reject = (_unit: undefined) => _ssrg_effect_fail(Invalid)

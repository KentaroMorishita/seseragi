import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { fail as _ssrg_effect_fail } from "@seseragi/runtime/effect"
$ssrg$assertUnicodeVersion("17.0.0")

export type AppError =
  | { readonly tag: "Invalid" };
export const Invalid: AppError = { tag: "Invalid" } as const;
export const reject = (_unit: undefined) => _ssrg_effect_fail(Invalid)

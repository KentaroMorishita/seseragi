import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { fail as _ssrg_effect_fail } from "@seseragi/runtime/effect"
const $ssrg$source56 = { file: "main.ssrg", line: 4, column: 24, uri: "seseragi://artifact/effect-fail-adt", start: 56, end: 68 };
$ssrg$assertUnicodeVersion("17.0.0")

export type AppError =
  | { readonly tag: "Invalid" };
export const Invalid: AppError = { tag: "Invalid" } as const;
export const reject = (_unit: undefined) => $ssrg$atSource($ssrg$source56, () => _ssrg_effect_fail(Invalid), true)

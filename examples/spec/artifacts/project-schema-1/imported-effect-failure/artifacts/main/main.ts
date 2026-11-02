import { type InputError, reject, __ssrg$instance$Show$0 as __ssrg$instance$Show$0_1 } from "./input.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { boundedShow as _ssrg_show_boundedShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError, succeed as _ssrg_effect_succeed } from "@seseragi/runtime/effect"
const $ssrg$source141 = { file: "project/imported-effect-failure/src/main.ssrg", line: 8, column: 5, uri: "seseragi://fixture/imported-effect-failure::main", start: 141, end: 180 };
const $ssrg$source164 = { file: "project/imported-effect-failure/src/main.ssrg", line: 8, column: 28, uri: "seseragi://fixture/imported-effect-failure::main", start: 164, end: 179 };
const $ssrg$source185 = { file: "project/imported-effect-failure/src/main.ssrg", line: 9, column: 5, uri: "seseragi://fixture/imported-effect-failure::main", start: 185, end: 195 };
$ssrg$assertUnicodeVersion("17.0.0")

export type AppError =
  | { readonly tag: "InputFailure"; readonly value: InputError };
export const InputFailure = (value: InputError): AppError => ({ tag: "InputFailure", value } as const);
export const __ssrg$instance$Show$0: _ssrg_show_Show<AppError> = _ssrg_show_boundedShow((value: AppError): string => { switch (value.tag) { case "InputFailure": return "InputFailure" + " " + __ssrg$instance$Show$0_1.show(value.value); } });
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source141, () => _ssrg_effect_mapError(InputFailure, $ssrg$atSource($ssrg$source164, () => reject("lizard"), true)), true), () => $ssrg$atSource($ssrg$source185, () => _ssrg_effect_succeed(undefined), true))

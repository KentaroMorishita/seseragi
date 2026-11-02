import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { boundedShow as _ssrg_show_boundedShow, stringShow as _ssrg_show_stringShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { fail as _ssrg_effect_fail } from "@seseragi/runtime/effect"
const $ssrg$source100 = { file: "project/imported-effect-failure/src/input.ssrg", line: 5, column: 3, uri: "seseragi://fixture/imported-effect-failure::input", start: 100, end: 125 };
const $ssrg$source106 = { file: "project/imported-effect-failure/src/input.ssrg", line: 5, column: 9, uri: "seseragi://fixture/imported-effect-failure::input", start: 106, end: 124 };
$ssrg$assertUnicodeVersion("17.0.0")

export type InputError =
  | { readonly tag: "InvalidInput"; readonly value: string };
export const InvalidInput = (value: string): InputError => ({ tag: "InvalidInput", value } as const);
export const __ssrg$instance$Show$0: _ssrg_show_Show<InputError> = _ssrg_show_boundedShow((value: InputError): string => { switch (value.tag) { case "InvalidInput": return "InvalidInput" + " " + _ssrg_show_stringShow.show(value.value); } });
export const reject = (input: string) => $ssrg$atSource($ssrg$source100, () => _ssrg_effect_fail($ssrg$atSource($ssrg$source106, () => InvalidInput(input), false)), true)

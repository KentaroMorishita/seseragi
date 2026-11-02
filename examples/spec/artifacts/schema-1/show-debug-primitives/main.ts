import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { Nothing as _ssrg_maybe_Nothing, Right as _ssrg_either_Right } from "@seseragi/runtime/sum"
import { intShow as _ssrg_show_intShow, intDebug as _ssrg_debug_intDebug, floatShow as _ssrg_show_floatShow, floatDebug as _ssrg_debug_floatDebug, neverShow as _ssrg_show_neverShow, neverDebug as _ssrg_debug_neverDebug, boolShow as _ssrg_show_boolShow, boolDebug as _ssrg_debug_boolDebug, unitShow as _ssrg_show_unitShow, unitDebug as _ssrg_debug_unitDebug, charShow as _ssrg_show_charShow, charDebug as _ssrg_debug_charDebug, stringDebug as _ssrg_debug_stringDebug, maybeShow as _ssrg_show_maybeShow, eitherDebug as _ssrg_debug_eitherDebug } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source40 = { file: "main.ssrg", line: 2, column: 3, uri: "seseragi://artifact/show-debug-primitives", start: 40, end: 50 };
const $ssrg$source93 = { file: "main.ssrg", line: 5, column: 3, uri: "seseragi://artifact/show-debug-primitives", start: 93, end: 104 };
const $ssrg$source150 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/show-debug-primitives", start: 150, end: 160 };
const $ssrg$source207 = { file: "main.ssrg", line: 11, column: 3, uri: "seseragi://artifact/show-debug-primitives", start: 207, end: 218 };
const $ssrg$source264 = { file: "main.ssrg", line: 14, column: 3, uri: "seseragi://artifact/show-debug-primitives", start: 264, end: 274 };
const $ssrg$source321 = { file: "main.ssrg", line: 17, column: 3, uri: "seseragi://artifact/show-debug-primitives", start: 321, end: 332 };
const $ssrg$source376 = { file: "main.ssrg", line: 20, column: 3, uri: "seseragi://artifact/show-debug-primitives", start: 376, end: 386 };
const $ssrg$source431 = { file: "main.ssrg", line: 23, column: 3, uri: "seseragi://artifact/show-debug-primitives", start: 431, end: 442 };
const $ssrg$source486 = { file: "main.ssrg", line: 26, column: 3, uri: "seseragi://artifact/show-debug-primitives", start: 486, end: 496 };
const $ssrg$source541 = { file: "main.ssrg", line: 29, column: 3, uri: "seseragi://artifact/show-debug-primitives", start: 541, end: 552 };
const $ssrg$source596 = { file: "main.ssrg", line: 32, column: 3, uri: "seseragi://artifact/show-debug-primitives", start: 596, end: 606 };
const $ssrg$source651 = { file: "main.ssrg", line: 35, column: 3, uri: "seseragi://artifact/show-debug-primitives", start: 651, end: 662 };
const $ssrg$source711 = { file: "main.ssrg", line: 38, column: 3, uri: "seseragi://artifact/show-debug-primitives", start: 711, end: 722 };
const $ssrg$source799 = { file: "main.ssrg", line: 42, column: 40, uri: "seseragi://artifact/show-debug-primitives", start: 799, end: 812 };
const $ssrg$source846 = { file: "main.ssrg", line: 46, column: 5, uri: "seseragi://artifact/show-debug-primitives", start: 846, end: 864 };
const $ssrg$source855 = { file: "main.ssrg", line: 46, column: 14, uri: "seseragi://artifact/show-debug-primitives", start: 855, end: 863 };
const $ssrg$source869 = { file: "main.ssrg", line: 47, column: 5, uri: "seseragi://artifact/show-debug-primitives", start: 869, end: 890 };
const $ssrg$source878 = { file: "main.ssrg", line: 47, column: 14, uri: "seseragi://artifact/show-debug-primitives", start: 878, end: 889 };
const $ssrg$source895 = { file: "main.ssrg", line: 48, column: 5, uri: "seseragi://artifact/show-debug-primitives", start: 895, end: 920 };
const $ssrg$source904 = { file: "main.ssrg", line: 48, column: 14, uri: "seseragi://artifact/show-debug-primitives", start: 904, end: 919 };
const $ssrg$source925 = { file: "main.ssrg", line: 49, column: 5, uri: "seseragi://artifact/show-debug-primitives", start: 925, end: 944 };
const $ssrg$source934 = { file: "main.ssrg", line: 49, column: 14, uri: "seseragi://artifact/show-debug-primitives", start: 934, end: 943 };
const $ssrg$source949 = { file: "main.ssrg", line: 50, column: 5, uri: "seseragi://artifact/show-debug-primitives", start: 949, end: 966 };
const $ssrg$source958 = { file: "main.ssrg", line: 50, column: 14, uri: "seseragi://artifact/show-debug-primitives", start: 958, end: 965 };
const $ssrg$source971 = { file: "main.ssrg", line: 51, column: 5, uri: "seseragi://artifact/show-debug-primitives", start: 971, end: 1005 };
const $ssrg$source980 = { file: "main.ssrg", line: 51, column: 14, uri: "seseragi://artifact/show-debug-primitives", start: 980, end: 1004 };
$ssrg$assertUnicodeVersion("17.0.0")

export const showInt = (value: number) => $ssrg$atSource($ssrg$source40, () => ((_ssrg_show_intShow["show"](value)) as string), false)
export const debugInt = (value: number) => $ssrg$atSource($ssrg$source93, () => ((_ssrg_debug_intDebug["debug"](value)) as string), false)
export const showFloat = (value: number) => $ssrg$atSource($ssrg$source150, () => ((_ssrg_show_floatShow["show"](value)) as string), false)
export const debugFloat = (value: number) => $ssrg$atSource($ssrg$source207, () => ((_ssrg_debug_floatDebug["debug"](value)) as string), false)
export const showNever = (value: never) => $ssrg$atSource($ssrg$source264, () => ((_ssrg_show_neverShow["show"](value)) as string), false)
export const debugNever = (value: never) => $ssrg$atSource($ssrg$source321, () => ((_ssrg_debug_neverDebug["debug"](value)) as string), false)
export const showBool = (value: boolean) => $ssrg$atSource($ssrg$source376, () => ((_ssrg_show_boolShow["show"](value)) as string), false)
export const debugBool = (value: boolean) => $ssrg$atSource($ssrg$source431, () => ((_ssrg_debug_boolDebug["debug"](value)) as string), false)
export const showUnit = (value: undefined) => $ssrg$atSource($ssrg$source486, () => ((_ssrg_show_unitShow["show"](value)) as string), false)
export const debugUnit = (value: undefined) => $ssrg$atSource($ssrg$source541, () => ((_ssrg_debug_unitDebug["debug"](value)) as string), false)
export const showChar = (value: string) => $ssrg$atSource($ssrg$source596, () => ((_ssrg_show_charShow["show"](value)) as string), false)
export const debugChar = (value: string) => $ssrg$atSource($ssrg$source651, () => ((_ssrg_debug_charDebug["debug"](value)) as string), false)
export const debugString = (value: string) => $ssrg$atSource($ssrg$source711, () => ((_ssrg_debug_stringDebug["debug"](value)) as string), false)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source846, () => _ssrg_console_println($ssrg$atSource($ssrg$source855, () => ((_ssrg_debug_intDebug["debug"](42)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source869, () => _ssrg_console_println($ssrg$atSource($ssrg$source878, () => ((_ssrg_show_maybeShow<never>(_ssrg_show_neverShow)["show"](absent)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source895, () => _ssrg_console_println($ssrg$atSource($ssrg$source904, () => ((_ssrg_debug_eitherDebug<never, string>(_ssrg_debug_neverDebug, _ssrg_debug_stringDebug)["debug"](available)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source925, () => _ssrg_console_println($ssrg$atSource($ssrg$source934, () => ((_ssrg_show_boolShow["show"](true)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source949, () => _ssrg_console_println($ssrg$atSource($ssrg$source958, () => ((_ssrg_show_unitShow["show"](undefined)) as string), false)), true), () => $ssrg$atSource($ssrg$source971, () => _ssrg_console_println($ssrg$atSource($ssrg$source980, () => ((_ssrg_debug_stringDebug["debug"]("line\n\"quoted\"")) as string), false)), true))))))
const absent: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: never } = _ssrg_maybe_Nothing;
const available: { readonly tag: "Left"; readonly value: never } | { readonly tag: "Right"; readonly value: string } = $ssrg$atSource($ssrg$source799, () => _ssrg_either_Right("ready"), false);

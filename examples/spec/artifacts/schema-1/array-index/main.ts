import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { index as _ssrg_array_index, get as _ssrg_array_get } from "@seseragi/runtime/array"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow, maybeShow as _ssrg_show_maybeShow, boolShow as _ssrg_show_boolShow } from "@seseragi/runtime/show"
import { subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { intEq as _ssrg_int_eq_dictionary, maybeEq as _ssrg_maybe_eq_dictionary } from "@seseragi/runtime/equality"
const $ssrg$source61 = { file: "main.ssrg", line: 1, column: 62, uri: "seseragi://artifact/array-index", start: 61, end: 75 };
const $ssrg$source176 = { file: "main.ssrg", line: 5, column: 12, uri: "seseragi://artifact/array-index", start: 176, end: 187 };
const $ssrg$source216 = { file: "main.ssrg", line: 7, column: 3, uri: "seseragi://artifact/array-index", start: 216, end: 240 };
const $ssrg$source226 = { file: "main.ssrg", line: 7, column: 13, uri: "seseragi://artifact/array-index", start: 226, end: 240 };
const $ssrg$source231 = { file: "main.ssrg", line: 7, column: 18, uri: "seseragi://artifact/array-index", start: 231, end: 240 };
const $ssrg$source243 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/array-index", start: 243, end: 267 };
const $ssrg$source253 = { file: "main.ssrg", line: 8, column: 13, uri: "seseragi://artifact/array-index", start: 253, end: 267 };
const $ssrg$source258 = { file: "main.ssrg", line: 8, column: 18, uri: "seseragi://artifact/array-index", start: 258, end: 267 };
const $ssrg$source270 = { file: "main.ssrg", line: 9, column: 3, uri: "seseragi://artifact/array-index", start: 270, end: 294 };
const $ssrg$source280 = { file: "main.ssrg", line: 9, column: 13, uri: "seseragi://artifact/array-index", start: 280, end: 294 };
const $ssrg$source285 = { file: "main.ssrg", line: 9, column: 18, uri: "seseragi://artifact/array-index", start: 285, end: 294 };
const $ssrg$source297 = { file: "main.ssrg", line: 10, column: 3, uri: "seseragi://artifact/array-index", start: 297, end: 322 };
const $ssrg$source307 = { file: "main.ssrg", line: 10, column: 13, uri: "seseragi://artifact/array-index", start: 307, end: 322 };
const $ssrg$source312 = { file: "main.ssrg", line: 10, column: 18, uri: "seseragi://artifact/array-index", start: 312, end: 322 };
const $ssrg$source319 = { file: "main.ssrg", line: 10, column: 25, uri: "seseragi://artifact/array-index", start: 319, end: 321 };
const $ssrg$source325 = { file: "main.ssrg", line: 11, column: 3, uri: "seseragi://artifact/array-index", start: 325, end: 349 };
const $ssrg$source335 = { file: "main.ssrg", line: 11, column: 13, uri: "seseragi://artifact/array-index", start: 335, end: 349 };
const $ssrg$source340 = { file: "main.ssrg", line: 11, column: 18, uri: "seseragi://artifact/array-index", start: 340, end: 349 };
const $ssrg$source352 = { file: "main.ssrg", line: 12, column: 3, uri: "seseragi://artifact/array-index", start: 352, end: 377 };
const $ssrg$source362 = { file: "main.ssrg", line: 12, column: 13, uri: "seseragi://artifact/array-index", start: 362, end: 377 };
const $ssrg$source367 = { file: "main.ssrg", line: 12, column: 18, uri: "seseragi://artifact/array-index", start: 367, end: 377 };
const $ssrg$source380 = { file: "main.ssrg", line: 13, column: 3, uri: "seseragi://artifact/array-index", start: 380, end: 403 };
const $ssrg$source390 = { file: "main.ssrg", line: 13, column: 13, uri: "seseragi://artifact/array-index", start: 390, end: 403 };
const $ssrg$source395 = { file: "main.ssrg", line: 13, column: 18, uri: "seseragi://artifact/array-index", start: 395, end: 403 };
const $ssrg$source406 = { file: "main.ssrg", line: 14, column: 3, uri: "seseragi://artifact/array-index", start: 406, end: 429 };
const $ssrg$source416 = { file: "main.ssrg", line: 14, column: 13, uri: "seseragi://artifact/array-index", start: 416, end: 429 };
const $ssrg$source422 = { file: "main.ssrg", line: 14, column: 19, uri: "seseragi://artifact/array-index", start: 422, end: 428 };
const $ssrg$source432 = { file: "main.ssrg", line: 15, column: 3, uri: "seseragi://artifact/array-index", start: 432, end: 462 };
const $ssrg$source442 = { file: "main.ssrg", line: 15, column: 13, uri: "seseragi://artifact/array-index", start: 442, end: 462 };
const $ssrg$source448 = { file: "main.ssrg", line: 15, column: 19, uri: "seseragi://artifact/array-index", start: 448, end: 461 };
const $ssrg$source465 = { file: "main.ssrg", line: 16, column: 3, uri: "seseragi://artifact/array-index", start: 465, end: 514 };
const $ssrg$source475 = { file: "main.ssrg", line: 16, column: 13, uri: "seseragi://artifact/array-index", start: 475, end: 514 };
const $ssrg$source481 = { file: "main.ssrg", line: 16, column: 19, uri: "seseragi://artifact/array-index", start: 481, end: 490 };
const $ssrg$source494 = { file: "main.ssrg", line: 16, column: 32, uri: "seseragi://artifact/array-index", start: 494, end: 513 };
$ssrg$assertUnicodeVersion("17.0.0")

export const pick = <A,>(values: ReadonlyArray<A>) => (offset: number) => $ssrg$atSource($ssrg$source61, () => _ssrg_array_index(values, offset), false)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source216, () => _ssrg_console_println($ssrg$atSource($ssrg$source226, () => ((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"]($ssrg$atSource($ssrg$source231, () => _ssrg_array_index(values, 0), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source243, () => _ssrg_console_println($ssrg$atSource($ssrg$source253, () => ((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"]($ssrg$atSource($ssrg$source258, () => _ssrg_array_index(values, 1), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source270, () => _ssrg_console_println($ssrg$atSource($ssrg$source280, () => ((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"]($ssrg$atSource($ssrg$source285, () => _ssrg_array_index(values, 2), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source297, () => _ssrg_console_println($ssrg$atSource($ssrg$source307, () => ((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"]($ssrg$atSource($ssrg$source312, () => _ssrg_array_index(values, _ssrg_int_subtract(0, 1, $ssrg$source319)), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source325, () => _ssrg_console_println($ssrg$atSource($ssrg$source335, () => ((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"]($ssrg$atSource($ssrg$source340, () => _ssrg_array_index(values, 3), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source352, () => _ssrg_console_println($ssrg$atSource($ssrg$source362, () => ((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"]($ssrg$atSource($ssrg$source367, () => _ssrg_array_index(values, 99), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source380, () => _ssrg_console_println($ssrg$atSource($ssrg$source390, () => ((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"]($ssrg$atSource($ssrg$source395, () => _ssrg_array_index(empty, 0), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source406, () => _ssrg_console_println($ssrg$atSource($ssrg$source416, () => ((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"]($ssrg$atSource($ssrg$source422, () => read(1), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source432, () => _ssrg_console_println($ssrg$atSource($ssrg$source442, () => ((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"]($ssrg$atSource($ssrg$source448, () => pick<number>([7, 8])(1), false))) as string), false)), true), () => $ssrg$atSource($ssrg$source465, () => _ssrg_console_println($ssrg$atSource($ssrg$source475, () => ((_ssrg_show_boolShow["show"]($ssrg$atSource($ssrg$source481, () => _ssrg_maybe_eq_dictionary<number>(_ssrg_int_eq_dictionary)["eq"]($ssrg$atSource($ssrg$source481, () => _ssrg_array_index(values, 1), false))($ssrg$atSource($ssrg$source494, () => _ssrg_array_get(1, values), false)), false))) as string), false)), true))))))))))
const values: ReadonlyArray<number> = [10, 20, 30];
const empty: ReadonlyArray<number> = [] as ReadonlyArray<number>;
const read: (argument: number) => { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number } = $ssrg$atSource($ssrg$source176, () => pick<number>(values), false);

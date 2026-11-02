import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { Cons as _ssrg_list_cons, fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow, maybeShow as _ssrg_show_maybeShow, listShow as _ssrg_show_listShow, boolShow as _ssrg_show_boolShow } from "@seseragi/runtime/show"
import { Nothing as _ssrg_maybe_Nothing, Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
import { intEq as _ssrg_int_eq_dictionary, listEq as _ssrg_list_eq_dictionary } from "@seseragi/runtime/equality"
const $ssrg$source52 = { file: "main.ssrg", line: 1, column: 53, uri: "seseragi://artifact/list-cons", start: 52, end: 63 };
const $ssrg$source179 = { file: "main.ssrg", line: 6, column: 25, uri: "seseragi://artifact/list-cons", start: 179, end: 194 };
const $ssrg$source183 = { file: "main.ssrg", line: 6, column: 29, uri: "seseragi://artifact/list-cons", start: 183, end: 194 };
const $ssrg$source187 = { file: "main.ssrg", line: 6, column: 33, uri: "seseragi://artifact/list-cons", start: 187, end: 194 };
const $ssrg$source210 = { file: "main.ssrg", line: 7, column: 16, uri: "seseragi://artifact/list-cons", start: 210, end: 215 };
const $ssrg$source244 = { file: "main.ssrg", line: 9, column: 3, uri: "seseragi://artifact/list-cons", start: 244, end: 280 };
const $ssrg$source254 = { file: "main.ssrg", line: 9, column: 13, uri: "seseragi://artifact/list-cons", start: 254, end: 280 };
const $ssrg$source260 = { file: "main.ssrg", line: 9, column: 19, uri: "seseragi://artifact/list-cons", start: 260, end: 279 };
const $ssrg$source272 = { file: "main.ssrg", line: 9, column: 31, uri: "seseragi://artifact/list-cons", start: 272, end: 278 };
const $ssrg$source283 = { file: "main.ssrg", line: 10, column: 3, uri: "seseragi://artifact/list-cons", start: 283, end: 304 };
const $ssrg$source293 = { file: "main.ssrg", line: 10, column: 13, uri: "seseragi://artifact/list-cons", start: 293, end: 304 };
const $ssrg$source307 = { file: "main.ssrg", line: 11, column: 3, uri: "seseragi://artifact/list-cons", start: 307, end: 340 };
const $ssrg$source317 = { file: "main.ssrg", line: 11, column: 13, uri: "seseragi://artifact/list-cons", start: 317, end: 340 };
const $ssrg$source323 = { file: "main.ssrg", line: 11, column: 19, uri: "seseragi://artifact/list-cons", start: 323, end: 339 };
const $ssrg$source343 = { file: "main.ssrg", line: 12, column: 3, uri: "seseragi://artifact/list-cons", start: 343, end: 372 };
const $ssrg$source353 = { file: "main.ssrg", line: 12, column: 13, uri: "seseragi://artifact/list-cons", start: 353, end: 372 };
const $ssrg$source359 = { file: "main.ssrg", line: 12, column: 19, uri: "seseragi://artifact/list-cons", start: 359, end: 371 };
const $ssrg$source375 = { file: "main.ssrg", line: 13, column: 3, uri: "seseragi://artifact/list-cons", start: 375, end: 424 };
const $ssrg$source385 = { file: "main.ssrg", line: 13, column: 13, uri: "seseragi://artifact/list-cons", start: 385, end: 424 };
const $ssrg$source391 = { file: "main.ssrg", line: 13, column: 19, uri: "seseragi://artifact/list-cons", start: 391, end: 406 };
$ssrg$assertUnicodeVersion("17.0.0")

const prepend = <A,>(head: A) => (tail: List<A>) => $ssrg$atSource($ssrg$source52, () => _ssrg_list_cons<A>(head, tail), false)
const first = (values: List<number>) => (($ssrg_match: List<number>): number => $ssrg_match.tag === "Cons" ? ((head: number, tail: List<number>): number => head)($ssrg_match.head, $ssrg_match.tail) : 0)(values)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source244, () => _ssrg_console_println($ssrg$atSource($ssrg$source254, () => ((_ssrg_show_listShow<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }>(_ssrg_show_maybeShow<number>(_ssrg_show_intShow))["show"]($ssrg$atSource($ssrg$source260, () => _ssrg_list_cons<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }>(_ssrg_maybe_Nothing, _ssrg_list_from_array([$ssrg$atSource($ssrg$source272, () => _ssrg_maybe_Just(1), false)])), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source283, () => _ssrg_console_println($ssrg$atSource($ssrg$source293, () => ((_ssrg_show_listShow<number>(_ssrg_show_intShow)["show"](values)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source307, () => _ssrg_console_println($ssrg$atSource($ssrg$source317, () => ((_ssrg_show_listShow<number>(_ssrg_show_intShow)["show"]($ssrg$atSource($ssrg$source323, () => prepend<number>(0)(values), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source343, () => _ssrg_console_println($ssrg$atSource($ssrg$source353, () => ((_ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source359, () => first(values), false))) as string), false)), true), () => $ssrg$atSource($ssrg$source375, () => _ssrg_console_println($ssrg$atSource($ssrg$source385, () => ((_ssrg_show_boolShow["show"]($ssrg$atSource($ssrg$source391, () => _ssrg_list_eq_dictionary<number>(_ssrg_int_eq_dictionary)["eq"]($ssrg$atSource($ssrg$source391, () => withFour(values), false))(_ssrg_list_from_array([4, 1, 2, 3])), false))) as string), false)), true)))))
const values: List<number> = $ssrg$atSource($ssrg$source179, () => _ssrg_list_cons<number>(1, $ssrg$atSource($ssrg$source183, () => _ssrg_list_cons<number>(2, $ssrg$atSource($ssrg$source187, () => _ssrg_list_cons<number>(3, _ssrg_list_from_array([] as ReadonlyArray<number>)), false)), false)), false);
const withFour: (argument: List<number>) => List<number> = $ssrg$atSource($ssrg$source210, () => (() => { const __ssrg$collection$saved0: number = 4; return (__ssrg$collection$partial$0: List<number>) => _ssrg_list_cons<number>(__ssrg$collection$saved0, __ssrg$collection$partial$0); })(), false);

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { fromArray as _ssrg_list_from_array, reduce as _ssrg_list_reduce, listFunctor as _ssrg_list_functor, listApplicative as _ssrg_list_applicative, listMonad as _ssrg_list_monad, type List as List } from "@seseragi/runtime/list"
import { stringShow as _ssrg_show_stringShow, intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source33 = { file: "main.ssrg", line: 1, column: 34, uri: "seseragi://artifact/list-monad", start: 33, end: 42 };
const $ssrg$source89 = { file: "main.ssrg", line: 3, column: 46, uri: "seseragi://artifact/list-monad", start: 89, end: 99 };
const $ssrg$source138 = { file: "main.ssrg", line: 5, column: 37, uri: "seseragi://artifact/list-monad", start: 138, end: 157 };
const $ssrg$source263 = { file: "main.ssrg", line: 11, column: 5, uri: "seseragi://artifact/list-monad", start: 263, end: 287 };
const $ssrg$source346 = { file: "main.ssrg", line: 16, column: 5, uri: "seseragi://artifact/list-monad", start: 346, end: 378 };
const $ssrg$source353 = { file: "main.ssrg", line: 16, column: 12, uri: "seseragi://artifact/list-monad", start: 353, end: 367 };
const $ssrg$source439 = { file: "main.ssrg", line: 21, column: 5, uri: "seseragi://artifact/list-monad", start: 439, end: 461 };
$ssrg$assertUnicodeVersion("17.0.0")

const increment = (value: number) => _ssrg_int_add(value, 1, $ssrg$source33)
const expand = (value: number) => _ssrg_list_from_array([value, _ssrg_int_add(value, 10, $ssrg$source89)])
const total = (values: List<number>) => $ssrg$atSource($ssrg$source138, () => _ssrg_list_reduce(0, (((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number), values), false)
const render = (label: string) => (value: number) => _ssrg_show_stringShow["show"](label) + ": " + _ssrg_show_intShow["show"](value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source263, () => _ssrg_console_println($ssrg$atSource($ssrg$source263, () => render("List map")($ssrg$atSource($ssrg$source263, () => total($ssrg$atSource($ssrg$source263, () => ((_ssrg_list_functor["map"](increment)(_ssrg_list_from_array([1, 2, 3]))) as List<number>), false)), false)), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source346, () => _ssrg_console_println($ssrg$atSource($ssrg$source346, () => render("List apply")($ssrg$atSource($ssrg$source346, () => total($ssrg$atSource($ssrg$source346, () => ((_ssrg_list_applicative["apply"]($ssrg$atSource($ssrg$source353, () => ((_ssrg_list_applicative["pure"](increment)) as List<(argument: number) => number>), false))(_ssrg_list_from_array([40, 41]))) as List<number>), false)), false)), false)), true), () => $ssrg$atSource($ssrg$source439, () => _ssrg_console_println($ssrg$atSource($ssrg$source439, () => render("List flatMap")($ssrg$atSource($ssrg$source439, () => total($ssrg$atSource($ssrg$source439, () => ((_ssrg_list_monad["flatMap"](expand)(_ssrg_list_from_array([1, 2]))) as List<number>), false)), false)), false)), true)))

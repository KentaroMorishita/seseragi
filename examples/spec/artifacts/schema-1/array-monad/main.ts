import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { reduce as _ssrg_array_reduce, arrayFunctor as _ssrg_array_functor, arrayApplicative as _ssrg_array_applicative, arrayMonad as _ssrg_array_monad } from "@seseragi/runtime/array"
import { stringShow as _ssrg_show_stringShow, intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source33 = { file: "main.ssrg", line: 1, column: 34, uri: "seseragi://artifact/array-monad", start: 33, end: 42 };
const $ssrg$source89 = { file: "main.ssrg", line: 3, column: 46, uri: "seseragi://artifact/array-monad", start: 89, end: 99 };
const $ssrg$source139 = { file: "main.ssrg", line: 5, column: 38, uri: "seseragi://artifact/array-monad", start: 139, end: 158 };
const $ssrg$source264 = { file: "main.ssrg", line: 11, column: 5, uri: "seseragi://artifact/array-monad", start: 264, end: 287 };
const $ssrg$source347 = { file: "main.ssrg", line: 16, column: 5, uri: "seseragi://artifact/array-monad", start: 347, end: 378 };
const $ssrg$source354 = { file: "main.ssrg", line: 16, column: 12, uri: "seseragi://artifact/array-monad", start: 354, end: 368 };
const $ssrg$source440 = { file: "main.ssrg", line: 21, column: 5, uri: "seseragi://artifact/array-monad", start: 440, end: 461 };
$ssrg$assertUnicodeVersion("17.0.0")

const increment = (value: number) => _ssrg_int_add(value, 1, $ssrg$source33)
const expand = (value: number) => [value, _ssrg_int_add(value, 10, $ssrg$source89)]
const total = (values: ReadonlyArray<number>) => $ssrg$atSource($ssrg$source139, () => _ssrg_array_reduce(0, (((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number), values), false)
const render = (label: string) => (value: number) => _ssrg_show_stringShow["show"](label) + ": " + _ssrg_show_intShow["show"](value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source264, () => _ssrg_console_println($ssrg$atSource($ssrg$source264, () => render("Array map")($ssrg$atSource($ssrg$source264, () => total($ssrg$atSource($ssrg$source264, () => ((_ssrg_array_functor["map"](increment)([1, 2, 3])) as ReadonlyArray<number>), false)), false)), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source347, () => _ssrg_console_println($ssrg$atSource($ssrg$source347, () => render("Array apply")($ssrg$atSource($ssrg$source347, () => total($ssrg$atSource($ssrg$source347, () => ((_ssrg_array_applicative["apply"]($ssrg$atSource($ssrg$source354, () => ((_ssrg_array_applicative["pure"](increment)) as ReadonlyArray<(argument: number) => number>), false))([40, 41])) as ReadonlyArray<number>), false)), false)), false)), true), () => $ssrg$atSource($ssrg$source440, () => _ssrg_console_println($ssrg$atSource($ssrg$source440, () => render("Array flatMap")($ssrg$atSource($ssrg$source440, () => total($ssrg$atSource($ssrg$source440, () => ((_ssrg_array_monad["flatMap"](expand)([1, 2])) as ReadonlyArray<number>), false)), false)), false)), true)))

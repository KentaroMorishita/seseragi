import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { reduce as _ssrg_array_reduce, arrayFunctor as _ssrg_array_functor, arrayApplicative as _ssrg_array_applicative, arrayMonad as _ssrg_array_monad } from "@seseragi/runtime/array"
import { stringShow as _ssrg_show_stringShow, intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

const increment = (value: number) => _ssrg_int_add(value, 1)
const expand = (value: number) => [value, _ssrg_int_add(value, 10)]
const total = (values: ReadonlyArray<number>) => _ssrg_array_reduce(0, (((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number), values)
const render = (label: string) => (value: number) => _ssrg_show_stringShow["show"](label) + ": " + _ssrg_show_intShow["show"](value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(render("Array map")(total(((_ssrg_array_functor["map"](increment)([1, 2, 3])) as ReadonlyArray<number>)))), () => _ssrg_effect_flatMap(_ssrg_console_println(render("Array apply")(total(((_ssrg_array_applicative["apply"](((_ssrg_array_applicative["pure"](increment)) as ReadonlyArray<(argument: number) => number>))([40, 41])) as ReadonlyArray<number>)))), () => _ssrg_console_println(render("Array flatMap")(total(((_ssrg_array_monad["flatMap"](expand)([1, 2])) as ReadonlyArray<number>))))))

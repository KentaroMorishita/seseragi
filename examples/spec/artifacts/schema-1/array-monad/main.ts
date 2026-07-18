import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { reduce as _ssrg_array_reduce, arrayFunctor as _ssrg_array_functor, arrayApplicative as _ssrg_array_applicative, arrayMonad as _ssrg_array_monad } from "@seseragi/runtime/array"
import { stringShow as _ssrg_show_stringShow, intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const increment = (value: bigint) => _ssrg_int64_add(value, 1n)
const expand = (value: bigint) => [value, _ssrg_int64_add(value, 10n)]
const total = (values: ReadonlyArray<bigint>) => _ssrg_array_reduce(0n, (_argument0) => (_argument1) => _ssrg_int64_add(_argument0, _argument1), values)
const render = (label: string) => (value: bigint) => _ssrg_show_stringShow["show"](label) + ": " + _ssrg_show_intShow["show"](value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(render("Array map")(total(_ssrg_array_functor["map"](increment)([1n, 2n, 3n])))), () => _ssrg_effect_flatMap(_ssrg_console_println(render("Array apply")(total(_ssrg_array_applicative["apply"](_ssrg_array_applicative["pure"](increment))([40n, 41n])))), () => _ssrg_console_println(render("Array flatMap")(total(_ssrg_array_monad["flatMap"](expand)([1n, 2n]))))))

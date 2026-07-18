import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { fromArray as _ssrg_list_from_array, reduce as _ssrg_list_reduce, listFunctor as _ssrg_list_functor, listApplicative as _ssrg_list_applicative, listMonad as _ssrg_list_monad, type List as List } from "@seseragi/runtime/list"
import { stringShow as _ssrg_show_stringShow, intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const increment = (value: bigint) => _ssrg_int64_add(value, 1n)
const expand = (value: bigint) => _ssrg_list_from_array([value, _ssrg_int64_add(value, 10n)])
const total = (values: List<bigint>) => _ssrg_list_reduce(0n, (_argument0) => (_argument1) => _ssrg_int64_add(_argument0, _argument1), values)
const render = (label: string) => (value: bigint) => _ssrg_show_stringShow["show"](label) + ": " + _ssrg_show_intShow["show"](value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(render("List map")(total(_ssrg_list_functor["map"](increment)(_ssrg_list_from_array([1n, 2n, 3n]))))), () => _ssrg_effect_flatMap(_ssrg_console_println(render("List apply")(total(_ssrg_list_applicative["apply"](_ssrg_list_applicative["pure"](increment))(_ssrg_list_from_array([40n, 41n]))))), () => _ssrg_console_println(render("List flatMap")(total(_ssrg_list_monad["flatMap"](expand)(_ssrg_list_from_array([1n, 2n])))))))

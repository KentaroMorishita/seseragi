import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { effectApplicative as _ssrg_effect_applicative, effectFunctor as _ssrg_effect_functor, effectMonad as _ssrg_effect_monad, flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { stringShow as _ssrg_show_stringShow, intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

const render = (label: string) => (value: number) => _ssrg_show_stringShow["show"](label) + ": " + _ssrg_show_intShow["show"](value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(mapped, (mappedValue: number) => _ssrg_effect_flatMap(_ssrg_console_println(render("Effect map")(mappedValue)), () => _ssrg_effect_flatMap(applied, (appliedValue: number) => _ssrg_effect_flatMap(_ssrg_console_println(render("Effect apply")(appliedValue)), () => _ssrg_effect_flatMap(chained, (chainedValue: number) => _ssrg_console_println(render("Effect flatMap")(chainedValue)))))))
const increment: (argument: number) => number = (value: number) => _ssrg_int_add(value, 1);
const source: Effect<{  }, never, number> = _ssrg_effect_applicative["pure"](41);
const liftedIncrement: Effect<{  }, never, (argument: number) => number> = _ssrg_effect_applicative["pure"](increment);
const plusTen: (argument: number) => Effect<{  }, never, number> = (value: number) => _ssrg_effect_applicative["pure"](_ssrg_int_add(value, 10));
const mapped: Effect<{  }, never, number> = _ssrg_effect_functor["map"](increment)(source);
const applied: Effect<{  }, never, number> = _ssrg_effect_applicative["apply"](liftedIncrement)(source);
const chained: Effect<{  }, never, number> = _ssrg_effect_monad["flatMap"](plusTen)(source);

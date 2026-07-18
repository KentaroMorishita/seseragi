import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { effectApplicative as _ssrg_effect_applicative, effectFunctor as _ssrg_effect_functor, effectMonad as _ssrg_effect_monad, flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { stringShow as _ssrg_show_stringShow, intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const increment: (argument: bigint) => bigint = (value: bigint) => _ssrg_int64_add(value, 1n);
const source: Effect<{  }, never, bigint> = _ssrg_effect_applicative["pure"](41n);
const liftedIncrement: Effect<{  }, never, (argument: bigint) => bigint> = _ssrg_effect_applicative["pure"](increment);
const plusTen: (argument: bigint) => Effect<{  }, never, bigint> = (value: bigint) => _ssrg_effect_applicative["pure"](_ssrg_int64_add(value, 10n));
const mapped: Effect<{  }, never, bigint> = _ssrg_effect_functor["map"](increment)(source);
const applied: Effect<{  }, never, bigint> = _ssrg_effect_applicative["apply"](liftedIncrement)(source);
const chained: Effect<{  }, never, bigint> = _ssrg_effect_monad["flatMap"](plusTen)(source);
const render = (label: string) => (value: bigint) => _ssrg_show_stringShow["show"](label) + ": " + _ssrg_show_intShow["show"](value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(mapped, (mappedValue: bigint) => _ssrg_effect_flatMap(_ssrg_console_println(render("Effect map")(mappedValue)), () => _ssrg_effect_flatMap(applied, (appliedValue: bigint) => _ssrg_effect_flatMap(_ssrg_console_println(render("Effect apply")(appliedValue)), () => _ssrg_effect_flatMap(chained, (chainedValue: bigint) => _ssrg_console_println(render("Effect flatMap")(chainedValue)))))))

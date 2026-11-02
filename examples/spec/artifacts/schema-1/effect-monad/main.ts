import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { effectApplicative as _ssrg_effect_applicative, effectFunctor as _ssrg_effect_functor, effectMonad as _ssrg_effect_monad, flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { stringShow as _ssrg_show_stringShow, intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source45 = { file: "main.ssrg", line: 1, column: 46, uri: "seseragi://artifact/effect-monad", start: 45, end: 54 };
const $ssrg$source93 = { file: "main.ssrg", line: 3, column: 38, uri: "seseragi://artifact/effect-monad", start: 93, end: 100 };
const $ssrg$source157 = { file: "main.ssrg", line: 5, column: 56, uri: "seseragi://artifact/effect-monad", start: 157, end: 171 };
const $ssrg$source237 = { file: "main.ssrg", line: 8, column: 18, uri: "seseragi://artifact/effect-monad", start: 237, end: 254 };
const $ssrg$source244 = { file: "main.ssrg", line: 8, column: 25, uri: "seseragi://artifact/effect-monad", start: 244, end: 254 };
const $ssrg$source293 = { file: "main.ssrg", line: 10, column: 38, uri: "seseragi://artifact/effect-monad", start: 293, end: 313 };
const $ssrg$source355 = { file: "main.ssrg", line: 13, column: 3, uri: "seseragi://artifact/effect-monad", start: 355, end: 383 };
const $ssrg$source423 = { file: "main.ssrg", line: 15, column: 39, uri: "seseragi://artifact/effect-monad", start: 423, end: 445 };
const $ssrg$source577 = { file: "main.ssrg", line: 22, column: 5, uri: "seseragi://artifact/effect-monad", start: 577, end: 618 };
const $ssrg$source587 = { file: "main.ssrg", line: 22, column: 15, uri: "seseragi://artifact/effect-monad", start: 587, end: 618 };
const $ssrg$source652 = { file: "main.ssrg", line: 25, column: 5, uri: "seseragi://artifact/effect-monad", start: 652, end: 696 };
const $ssrg$source662 = { file: "main.ssrg", line: 25, column: 15, uri: "seseragi://artifact/effect-monad", start: 662, end: 696 };
const $ssrg$source730 = { file: "main.ssrg", line: 28, column: 5, uri: "seseragi://artifact/effect-monad", start: 730, end: 776 };
const $ssrg$source740 = { file: "main.ssrg", line: 28, column: 15, uri: "seseragi://artifact/effect-monad", start: 740, end: 776 };
$ssrg$assertUnicodeVersion("17.0.0")

const render = (label: string) => (value: number) => _ssrg_show_stringShow["show"](label) + ": " + _ssrg_show_intShow["show"](value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(mapped, (mappedValue: number) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source577, () => _ssrg_console_println($ssrg$atSource($ssrg$source587, () => render("Effect map")(mappedValue), false)), true), () => _ssrg_effect_flatMap(applied, (appliedValue: number) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source652, () => _ssrg_console_println($ssrg$atSource($ssrg$source662, () => render("Effect apply")(appliedValue), false)), true), () => _ssrg_effect_flatMap(chained, (chainedValue: number) => $ssrg$atSource($ssrg$source730, () => _ssrg_console_println($ssrg$atSource($ssrg$source740, () => render("Effect flatMap")(chainedValue), false)), true))))))
const increment: (argument: number) => number = (value: number) => _ssrg_int_add(value, 1, $ssrg$source45);
const source: Effect<{  }, never, number> = $ssrg$atSource($ssrg$source93, () => ((_ssrg_effect_applicative["pure"](41)) as Effect<{  }, never, number>), true);
const liftedIncrement: Effect<{  }, never, (argument: number) => number> = $ssrg$atSource($ssrg$source157, () => ((_ssrg_effect_applicative["pure"](increment)) as Effect<{  }, never, (argument: number) => number>), true);
const plusTen: (argument: number) => Effect<{  }, never, number> = (value: number) => $ssrg$atSource($ssrg$source237, () => ((_ssrg_effect_applicative["pure"](_ssrg_int_add(value, 10, $ssrg$source244))) as Effect<{  }, never, number>), true);
const mapped: Effect<{  }, never, number> = $ssrg$atSource($ssrg$source293, () => ((_ssrg_effect_functor["map"](increment)(source)) as Effect<{  }, never, number>), true);
const applied: Effect<{  }, never, number> = $ssrg$atSource($ssrg$source355, () => ((_ssrg_effect_applicative["apply"](liftedIncrement)(source)) as Effect<{  }, never, number>), true);
const chained: Effect<{  }, never, number> = $ssrg$atSource($ssrg$source423, () => ((_ssrg_effect_monad["flatMap"](plusTen)(source)) as Effect<{  }, never, number>), true);

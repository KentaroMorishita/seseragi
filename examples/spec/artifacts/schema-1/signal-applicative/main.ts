import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { make as _ssrg_signal_make, map as _ssrg_signal_map, signalFunctor as _ssrg_signal_functor, constant as _ssrg_signal_constant, signalApplicative as _ssrg_signal_applicative, subscribe as _ssrg_signal_subscribe, set as _ssrg_signal_set, unsubscribe as _ssrg_signal_unsubscribe, read as _ssrg_signal_read, type MutableSignal as MutableSignal, type Signal as Signal, type Subscription as Subscription } from "@seseragi/runtime/signal"
import { multiply as _ssrg_int_multiply, add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
const $ssrg$source121 = { file: "main.ssrg", line: 7, column: 15, uri: "seseragi://artifact/signal-applicative", start: 121, end: 136 };
const $ssrg$source151 = { file: "main.ssrg", line: 8, column: 15, uri: "seseragi://artifact/signal-applicative", start: 151, end: 165 };
const $ssrg$source183 = { file: "main.ssrg", line: 9, column: 18, uri: "seseragi://artifact/signal-applicative", start: 183, end: 224 };
const $ssrg$source243 = { file: "main.ssrg", line: 10, column: 19, uri: "seseragi://artifact/signal-applicative", start: 243, end: 280 };
const $ssrg$source259 = { file: "main.ssrg", line: 10, column: 35, uri: "seseragi://artifact/signal-applicative", start: 259, end: 268 };
const $ssrg$source301 = { file: "main.ssrg", line: 11, column: 21, uri: "seseragi://artifact/signal-applicative", start: 301, end: 344 };
const $ssrg$source334 = { file: "main.ssrg", line: 11, column: 54, uri: "seseragi://artifact/signal-applicative", start: 334, end: 343 };
const $ssrg$source362 = { file: "main.ssrg", line: 12, column: 18, uri: "seseragi://artifact/signal-applicative", start: 362, end: 383 };
const $ssrg$source405 = { file: "main.ssrg", line: 14, column: 21, uri: "seseragi://artifact/signal-applicative", start: 405, end: 462 };
const $ssrg$source439 = { file: "main.ssrg", line: 14, column: 55, uri: "seseragi://artifact/signal-applicative", start: 439, end: 454 };
const $ssrg$source467 = { file: "main.ssrg", line: 15, column: 5, uri: "seseragi://artifact/signal-applicative", start: 467, end: 479 };
const $ssrg$source484 = { file: "main.ssrg", line: 16, column: 5, uri: "seseragi://artifact/signal-applicative", start: 484, end: 516 };
const $ssrg$source533 = { file: "main.ssrg", line: 18, column: 16, uri: "seseragi://artifact/signal-applicative", start: 533, end: 540 };
const $ssrg$source545 = { file: "main.ssrg", line: 19, column: 5, uri: "seseragi://artifact/signal-applicative", start: 545, end: 597 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source121, () => _ssrg_signal_make<number>(10), true), (source: MutableSignal<number>) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source151, () => _ssrg_signal_make<number>(0), true), (mirror: MutableSignal<number>) => (() => { const values: Signal<number> = $ssrg$atSource($ssrg$source183, () => _ssrg_signal_map((value: number) => value, source), false); return (() => { const doubled: Signal<number> = $ssrg$atSource($ssrg$source243, () => ((_ssrg_signal_functor["map"]((value: number) => _ssrg_int_multiply(value, 2, $ssrg$source259))(values)) as Signal<number>), false); return (() => { const functions: Signal<(argument: number) => number> = $ssrg$atSource($ssrg$source301, () => _ssrg_signal_constant((value: number) => _ssrg_int_add(value, 2, $ssrg$source334)), false); return (() => { const answer: Signal<number> = $ssrg$atSource($ssrg$source362, () => ((_ssrg_signal_applicative["apply"](functions)(doubled)) as Signal<number>), false); return _ssrg_effect_flatMap($ssrg$atSource($ssrg$source405, () => _ssrg_signal_subscribe((value: number) => $ssrg$atSource($ssrg$source439, () => _ssrg_signal_set(value, mirror), true), answer), true), (subscription: Subscription) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source467, () => _ssrg_signal_set(20, source), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source484, () => _ssrg_signal_unsubscribe(subscription), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source533, () => _ssrg_signal_read(mirror), true), (current: number) => $ssrg$atSource($ssrg$source545, () => _ssrg_console_println("Signal Functor / Applicative: " + _ssrg_show_intShow["show"](current)), true))))); })(); })(); })(); })()))

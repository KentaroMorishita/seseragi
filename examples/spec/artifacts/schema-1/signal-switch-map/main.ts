import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { make as _ssrg_signal_make, switchMap as _ssrg_signal_switchMap, constant as _ssrg_signal_constant, subscribe as _ssrg_signal_subscribe, set as _ssrg_signal_set, unsubscribe as _ssrg_signal_unsubscribe, read as _ssrg_signal_read, type MutableSignal as MutableSignal, type Signal as Signal, type Subscription as Subscription } from "@seseragi/runtime/signal"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
const $ssrg$source121 = { file: "main.ssrg", line: 7, column: 15, uri: "seseragi://artifact/signal-switch-map", start: 121, end: 136 };
const $ssrg$source151 = { file: "main.ssrg", line: 8, column: 15, uri: "seseragi://artifact/signal-switch-map", start: 151, end: 165 };
const $ssrg$source185 = { file: "main.ssrg", line: 9, column: 20, uri: "seseragi://artifact/signal-switch-map", start: 185, end: 255 };
const $ssrg$source219 = { file: "main.ssrg", line: 9, column: 54, uri: "seseragi://artifact/signal-switch-map", start: 219, end: 247 };
const $ssrg$source237 = { file: "main.ssrg", line: 9, column: 72, uri: "seseragi://artifact/signal-switch-map", start: 237, end: 246 };
const $ssrg$source277 = { file: "main.ssrg", line: 11, column: 21, uri: "seseragi://artifact/signal-switch-map", start: 277, end: 336 };
const $ssrg$source311 = { file: "main.ssrg", line: 11, column: 55, uri: "seseragi://artifact/signal-switch-map", start: 311, end: 326 };
const $ssrg$source341 = { file: "main.ssrg", line: 12, column: 5, uri: "seseragi://artifact/signal-switch-map", start: 341, end: 353 };
const $ssrg$source358 = { file: "main.ssrg", line: 13, column: 5, uri: "seseragi://artifact/signal-switch-map", start: 358, end: 390 };
const $ssrg$source407 = { file: "main.ssrg", line: 15, column: 16, uri: "seseragi://artifact/signal-switch-map", start: 407, end: 414 };
const $ssrg$source419 = { file: "main.ssrg", line: 16, column: 5, uri: "seseragi://artifact/signal-switch-map", start: 419, end: 452 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source121, () => _ssrg_signal_make<number>(10), true), (source: MutableSignal<number>) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source151, () => _ssrg_signal_make<number>(0), true), (mirror: MutableSignal<number>) => (() => { const selected: Signal<number> = $ssrg$atSource($ssrg$source185, () => _ssrg_signal_switchMap((value: number) => $ssrg$atSource($ssrg$source219, () => _ssrg_signal_constant(_ssrg_int_add(value, 1, $ssrg$source237)), false), source), false); return _ssrg_effect_flatMap($ssrg$atSource($ssrg$source277, () => _ssrg_signal_subscribe((value: number) => $ssrg$atSource($ssrg$source311, () => _ssrg_signal_set(value, mirror), true), selected), true), (subscription: Subscription) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source341, () => _ssrg_signal_set(41, source), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source358, () => _ssrg_signal_unsubscribe(subscription), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source407, () => _ssrg_signal_read(mirror), true), (current: number) => $ssrg$atSource($ssrg$source419, () => _ssrg_console_println("switchMap: " + _ssrg_show_intShow["show"](current)), true))))); })()))

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { make as _ssrg_signal_make, combine as _ssrg_signal_combine, subscribe as _ssrg_signal_subscribe, set as _ssrg_signal_set, transaction as _ssrg_signal_transaction, planSet as _ssrg_signal_planSet, unsubscribe as _ssrg_signal_unsubscribe, read as _ssrg_signal_read, type MutableSignal as MutableSignal, type Signal as Signal, type Subscription as Subscription, type SignalChange as SignalChange } from "@seseragi/runtime/signal"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
const $ssrg$source79 = { file: "main.ssrg", line: 3, column: 41, uri: "seseragi://artifact/signal-subscription", start: 79, end: 91 };
const $ssrg$source173 = { file: "main.ssrg", line: 9, column: 13, uri: "seseragi://artifact/signal-subscription", start: 173, end: 187 };
const $ssrg$source201 = { file: "main.ssrg", line: 10, column: 14, uri: "seseragi://artifact/signal-subscription", start: 201, end: 215 };
const $ssrg$source230 = { file: "main.ssrg", line: 11, column: 15, uri: "seseragi://artifact/signal-subscription", start: 230, end: 244 };
const $ssrg$source261 = { file: "main.ssrg", line: 12, column: 17, uri: "seseragi://artifact/signal-subscription", start: 261, end: 291 };
const $ssrg$source313 = { file: "main.ssrg", line: 14, column: 21, uri: "seseragi://artifact/signal-subscription", start: 313, end: 369 };
const $ssrg$source347 = { file: "main.ssrg", line: 14, column: 55, uri: "seseragi://artifact/signal-subscription", start: 347, end: 362 };
const $ssrg$source374 = { file: "main.ssrg", line: 15, column: 5, uri: "seseragi://artifact/signal-subscription", start: 374, end: 464 };
const $ssrg$source402 = { file: "main.ssrg", line: 16, column: 7, uri: "seseragi://artifact/signal-subscription", start: 402, end: 425 };
const $ssrg$source433 = { file: "main.ssrg", line: 17, column: 7, uri: "seseragi://artifact/signal-subscription", start: 433, end: 457 };
const $ssrg$source469 = { file: "main.ssrg", line: 19, column: 5, uri: "seseragi://artifact/signal-subscription", start: 469, end: 501 };
const $ssrg$source506 = { file: "main.ssrg", line: 20, column: 5, uri: "seseragi://artifact/signal-subscription", start: 506, end: 517 };
const $ssrg$source534 = { file: "main.ssrg", line: 22, column: 16, uri: "seseragi://artifact/signal-subscription", start: 534, end: 541 };
const $ssrg$source546 = { file: "main.ssrg", line: 23, column: 5, uri: "seseragi://artifact/signal-subscription", start: 546, end: 582 };
$ssrg$assertUnicodeVersion("17.0.0")

const add = (left: number) => (right: number) => _ssrg_int_add(left, right, $ssrg$source79)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source173, () => _ssrg_signal_make<number>(1), true), (left: MutableSignal<number>) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source201, () => _ssrg_signal_make<number>(2), true), (right: MutableSignal<number>) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source230, () => _ssrg_signal_make<number>(0), true), (mirror: MutableSignal<number>) => (() => { const total: Signal<number> = $ssrg$atSource($ssrg$source261, () => _ssrg_signal_combine(add, left, right), false); return _ssrg_effect_flatMap($ssrg$atSource($ssrg$source313, () => _ssrg_signal_subscribe((value: number) => $ssrg$atSource($ssrg$source347, () => _ssrg_signal_set(value, mirror), true), total), true), (subscription: Subscription) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source374, () => _ssrg_signal_transaction([$ssrg$atSource($ssrg$source402, () => _ssrg_signal_planSet(10, left), false), $ssrg$atSource($ssrg$source433, () => _ssrg_signal_planSet(20, right), false)]), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source469, () => _ssrg_signal_unsubscribe(subscription), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source506, () => _ssrg_signal_set(100, left), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source534, () => _ssrg_signal_read(mirror), true), (current: number) => $ssrg$atSource($ssrg$source546, () => _ssrg_console_println("subscription: " + _ssrg_show_intShow["show"](current)), true)))))); })())))

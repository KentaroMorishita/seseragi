import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { make as _ssrg_signal_make, map as _ssrg_signal_map, combine as _ssrg_signal_combine, planSet as _ssrg_signal_planSet, planUpdate as _ssrg_signal_planUpdate, set as _ssrg_signal_set, transaction as _ssrg_signal_transaction, read as _ssrg_signal_read, type MutableSignal as MutableSignal, type Signal as Signal, type SignalChange as SignalChange } from "@seseragi/runtime/signal"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
const $ssrg$source68 = { file: "main.ssrg", line: 3, column: 30, uri: "seseragi://artifact/signal-transaction", start: 68, end: 81 };
const $ssrg$source122 = { file: "main.ssrg", line: 4, column: 41, uri: "seseragi://artifact/signal-transaction", start: 122, end: 134 };
const $ssrg$source216 = { file: "main.ssrg", line: 10, column: 13, uri: "seseragi://artifact/signal-transaction", start: 216, end: 231 };
const $ssrg$source245 = { file: "main.ssrg", line: 11, column: 14, uri: "seseragi://artifact/signal-transaction", start: 245, end: 259 };
const $ssrg$source278 = { file: "main.ssrg", line: 12, column: 19, uri: "seseragi://artifact/signal-transaction", start: 278, end: 300 };
const $ssrg$source317 = { file: "main.ssrg", line: 13, column: 17, uri: "seseragi://artifact/signal-transaction", start: 317, end: 350 };
const $ssrg$source369 = { file: "main.ssrg", line: 14, column: 19, uri: "seseragi://artifact/signal-transaction", start: 369, end: 387 };
const $ssrg$source410 = { file: "main.ssrg", line: 15, column: 23, uri: "seseragi://artifact/signal-transaction", start: 410, end: 456 };
const $ssrg$source445 = { file: "main.ssrg", line: 15, column: 58, uri: "seseragi://artifact/signal-transaction", start: 445, end: 455 };
const $ssrg$source462 = { file: "main.ssrg", line: 17, column: 5, uri: "seseragi://artifact/signal-transaction", start: 462, end: 472 };
const $ssrg$source477 = { file: "main.ssrg", line: 18, column: 5, uri: "seseragi://artifact/signal-transaction", start: 477, end: 549 };
const $ssrg$source505 = { file: "main.ssrg", line: 19, column: 7, uri: "seseragi://artifact/signal-transaction", start: 505, end: 517 };
const $ssrg$source525 = { file: "main.ssrg", line: 20, column: 7, uri: "seseragi://artifact/signal-transaction", start: 525, end: 542 };
const $ssrg$source566 = { file: "main.ssrg", line: 23, column: 16, uri: "seseragi://artifact/signal-transaction", start: 566, end: 572 };
const $ssrg$source577 = { file: "main.ssrg", line: 24, column: 5, uri: "seseragi://artifact/signal-transaction", start: 577, end: 607 };
$ssrg$assertUnicodeVersion("17.0.0")

const twice = (value: number) => _ssrg_int_add(value, value, $ssrg$source68)
const add = (left: number) => (right: number) => _ssrg_int_add(left, right, $ssrg$source122)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source216, () => _ssrg_signal_make<number>(20), true), (left: MutableSignal<number>) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source245, () => _ssrg_signal_make<number>(1), true), (right: MutableSignal<number>) => (() => { const doubled: Signal<number> = $ssrg$atSource($ssrg$source278, () => _ssrg_signal_map(twice, left), false); return (() => { const total: Signal<number> = $ssrg$atSource($ssrg$source317, () => _ssrg_signal_combine(add, doubled, right), false); return (() => { const setLeft: (argument: MutableSignal<number>) => SignalChange = $ssrg$atSource($ssrg$source369, () => _ssrg_signal_planSet(21), false); return (() => { const updateRight: (argument: MutableSignal<number>) => SignalChange = $ssrg$atSource($ssrg$source410, () => _ssrg_signal_planUpdate((value: number) => _ssrg_int_add(value, 20, $ssrg$source445)), false); return _ssrg_effect_flatMap($ssrg$atSource($ssrg$source462, () => _ssrg_signal_set(10, left), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source477, () => _ssrg_signal_transaction([$ssrg$atSource($ssrg$source505, () => setLeft(left), false), $ssrg$atSource($ssrg$source525, () => updateRight(right), false)]), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source566, () => _ssrg_signal_read(total), true), (current: number) => $ssrg$atSource($ssrg$source577, () => _ssrg_console_println("signal: " + _ssrg_show_intShow["show"](current)), true)))); })(); })(); })(); })()))

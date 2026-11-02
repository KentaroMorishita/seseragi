import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { remainder as _ssrg_int_remainder, add as _ssrg_int_add } from "@seseragi/runtime/int"
import { flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { make as _ssrg_signal_make, distinct as _ssrg_signal_distinct, subscribe as _ssrg_signal_subscribe, update as _ssrg_signal_update, transaction as _ssrg_signal_transaction, planSet as _ssrg_signal_planSet, set as _ssrg_signal_set, unsubscribe as _ssrg_signal_unsubscribe, read as _ssrg_signal_read, type MutableSignal as MutableSignal, type Signal as Signal, type Subscription as Subscription, type SignalChange as SignalChange } from "@seseragi/runtime/signal"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
const $ssrg$source150 = { file: "main.ssrg", line: 9, column: 5, uri: "seseragi://artifact/signal-distinct", start: 150, end: 164 };
const $ssrg$source168 = { file: "main.ssrg", line: 9, column: 23, uri: "seseragi://artifact/signal-distinct", start: 168, end: 183 };
const $ssrg$source269 = { file: "main.ssrg", line: 16, column: 15, uri: "seseragi://artifact/signal-distinct", start: 269, end: 304 };
const $ssrg$source326 = { file: "main.ssrg", line: 17, column: 22, uri: "seseragi://artifact/signal-distinct", start: 326, end: 340 };
const $ssrg$source360 = { file: "main.ssrg", line: 18, column: 20, uri: "seseragi://artifact/signal-distinct", start: 360, end: 383 };
const $ssrg$source404 = { file: "main.ssrg", line: 19, column: 21, uri: "seseragi://artifact/signal-distinct", start: 404, end: 516 };
const $ssrg$source451 = { file: "main.ssrg", line: 20, column: 7, uri: "seseragi://artifact/signal-distinct", start: 451, end: 506 };
const $ssrg$source482 = { file: "main.ssrg", line: 20, column: 38, uri: "seseragi://artifact/signal-distinct", start: 482, end: 491 };
const $ssrg$source522 = { file: "main.ssrg", line: 22, column: 5, uri: "seseragi://artifact/signal-distinct", start: 522, end: 655 };
const $ssrg$source550 = { file: "main.ssrg", line: 23, column: 7, uri: "seseragi://artifact/signal-distinct", start: 550, end: 595 };
const $ssrg$source603 = { file: "main.ssrg", line: 24, column: 7, uri: "seseragi://artifact/signal-distinct", start: 603, end: 648 };
const $ssrg$source660 = { file: "main.ssrg", line: 26, column: 5, uri: "seseragi://artifact/signal-distinct", start: 660, end: 690 };
const $ssrg$source695 = { file: "main.ssrg", line: 27, column: 5, uri: "seseragi://artifact/signal-distinct", start: 695, end: 725 };
const $ssrg$source730 = { file: "main.ssrg", line: 28, column: 5, uri: "seseragi://artifact/signal-distinct", start: 730, end: 760 };
const $ssrg$source765 = { file: "main.ssrg", line: 29, column: 5, uri: "seseragi://artifact/signal-distinct", start: 765, end: 797 };
const $ssrg$source811 = { file: "main.ssrg", line: 30, column: 14, uri: "seseragi://artifact/signal-distinct", start: 811, end: 825 };
const $ssrg$source841 = { file: "main.ssrg", line: 31, column: 16, uri: "seseragi://artifact/signal-distinct", start: 841, end: 850 };
const $ssrg$source855 = { file: "main.ssrg", line: 32, column: 5, uri: "seseragi://artifact/signal-distinct", start: 855, end: 904 };
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Reading: unique symbol;
type Reading = {
  readonly "value": number;
  readonly [__ssrg$brand$Reading]: true;
};
export const __ssrg$instance$Eq$0 = { "eq": (left: Reading) => (right: Reading) => $ssrg$atSource($ssrg$source150, () => _ssrg_int_eq_dictionary["eq"](_ssrg_int_remainder((left)["value"], 2, $ssrg$source150))(_ssrg_int_remainder((right)["value"], 2, $ssrg$source168)), false) } as const;
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source269, () => _ssrg_signal_make<Reading>((({ "value": 0 } as const) as unknown as Reading)), true), (source: MutableSignal<Reading>) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source326, () => _ssrg_signal_make<number>(0), true), (notifications: MutableSignal<number>) => (() => { const filtered: Signal<Reading> = $ssrg$atSource($ssrg$source360, () => _ssrg_signal_distinct(__ssrg$instance$Eq$0["eq"], source), false); return _ssrg_effect_flatMap($ssrg$atSource($ssrg$source404, () => _ssrg_signal_subscribe((_reading: Reading) => $ssrg$atSource($ssrg$source451, () => _ssrg_signal_update((count: number) => _ssrg_int_add(count, 1, $ssrg$source482), notifications), true), filtered), true), (subscription: Subscription) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source522, () => _ssrg_signal_transaction([$ssrg$atSource($ssrg$source550, () => _ssrg_signal_planSet((({ "value": 1 } as const) as unknown as Reading), source), false), $ssrg$atSource($ssrg$source603, () => _ssrg_signal_planSet((({ "value": 2 } as const) as unknown as Reading), source), false)]), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source660, () => _ssrg_signal_set((({ "value": 3 } as const) as unknown as Reading), source), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source695, () => _ssrg_signal_set((({ "value": 5 } as const) as unknown as Reading), source), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source730, () => _ssrg_signal_set((({ "value": 6 } as const) as unknown as Reading), source), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source765, () => _ssrg_signal_unsubscribe(subscription), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source811, () => _ssrg_signal_read(notifications), true), (count: number) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source841, () => _ssrg_signal_read(filtered), true), (current: Reading) => $ssrg$atSource($ssrg$source855, () => _ssrg_console_println("distinct: " + _ssrg_show_intShow["show"](count) + " / " + _ssrg_show_intShow["show"]((current)["value"])), true))))))))); })()))

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { make as _ssrg_signal_make, combine as _ssrg_signal_combine, subscribe as _ssrg_signal_subscribe, set as _ssrg_signal_set, transaction as _ssrg_signal_transaction, planSet as _ssrg_signal_planSet, unsubscribe as _ssrg_signal_unsubscribe, read as _ssrg_signal_read, type MutableSignal as MutableSignal, type Signal as Signal, type Subscription as Subscription, type SignalChange as SignalChange } from "@seseragi/runtime/signal"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

const add = (left: number) => (right: number) => _ssrg_int_add(left, right)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_signal_make<number>(1), (left: MutableSignal<number>) => _ssrg_effect_flatMap(_ssrg_signal_make<number>(2), (right: MutableSignal<number>) => _ssrg_effect_flatMap(_ssrg_signal_make<number>(0), (mirror: MutableSignal<number>) => (() => { const total: Signal<number> = _ssrg_signal_combine(add, left, right); return _ssrg_effect_flatMap(_ssrg_signal_subscribe((value: number) => _ssrg_signal_set(value, mirror), total), (subscription: Subscription) => _ssrg_effect_flatMap(_ssrg_signal_transaction([_ssrg_signal_planSet(10, left), _ssrg_signal_planSet(20, right)]), () => _ssrg_effect_flatMap(_ssrg_signal_unsubscribe(subscription), () => _ssrg_effect_flatMap(_ssrg_signal_set(100, left), () => _ssrg_effect_flatMap(_ssrg_signal_read(mirror), (current: number) => _ssrg_console_println("subscription: " + _ssrg_show_intShow["show"](current))))))); })())))

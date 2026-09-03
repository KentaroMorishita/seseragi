import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { remainder as _ssrg_int_remainder, add as _ssrg_int_add } from "@seseragi/runtime/int"
import { flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { make as _ssrg_signal_make, distinct as _ssrg_signal_distinct, subscribe as _ssrg_signal_subscribe, update as _ssrg_signal_update, transaction as _ssrg_signal_transaction, planSet as _ssrg_signal_planSet, set as _ssrg_signal_set, unsubscribe as _ssrg_signal_unsubscribe, read as _ssrg_signal_read, type MutableSignal as MutableSignal, type Signal as Signal, type Subscription as Subscription, type SignalChange as SignalChange } from "@seseragi/runtime/signal"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Reading: unique symbol;
type Reading = {
  readonly "value": number;
  readonly [__ssrg$brand$Reading]: true;
};
export const __ssrg$instance$Eq$0 = { "eq": (left: Reading) => (right: Reading) => _ssrg_int_eq_dictionary["eq"](_ssrg_int_remainder((left)["value"], 2))(_ssrg_int_remainder((right)["value"], 2)) } as const;
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_signal_make<Reading>((({ "value": 0 } as const) as unknown as Reading)), (source: MutableSignal<Reading>) => _ssrg_effect_flatMap(_ssrg_signal_make<number>(0), (notifications: MutableSignal<number>) => (() => { const filtered: Signal<Reading> = _ssrg_signal_distinct((_argument0) => (_argument1) => __ssrg$instance$Eq$0["eq"](_argument0)(_argument1), source); return _ssrg_effect_flatMap(_ssrg_signal_subscribe((_reading: Reading) => _ssrg_signal_update((count: number) => _ssrg_int_add(count, 1), notifications), filtered), (subscription: Subscription) => _ssrg_effect_flatMap(_ssrg_signal_transaction([_ssrg_signal_planSet((({ "value": 1 } as const) as unknown as Reading), source), _ssrg_signal_planSet((({ "value": 2 } as const) as unknown as Reading), source)]), () => _ssrg_effect_flatMap(_ssrg_signal_set((({ "value": 3 } as const) as unknown as Reading), source), () => _ssrg_effect_flatMap(_ssrg_signal_set((({ "value": 5 } as const) as unknown as Reading), source), () => _ssrg_effect_flatMap(_ssrg_signal_set((({ "value": 6 } as const) as unknown as Reading), source), () => _ssrg_effect_flatMap(_ssrg_signal_unsubscribe(subscription), () => _ssrg_effect_flatMap(_ssrg_signal_read(notifications), (count: number) => _ssrg_effect_flatMap(_ssrg_signal_read(filtered), (current: Reading) => _ssrg_console_println("distinct: " + _ssrg_show_intShow["show"](count) + " / " + _ssrg_show_intShow["show"]((current)["value"])))))))))); })()))

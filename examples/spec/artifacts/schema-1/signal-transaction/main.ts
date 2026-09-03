import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { make as _ssrg_signal_make, map as _ssrg_signal_map, combine as _ssrg_signal_combine, planSet as _ssrg_signal_planSet, planUpdate as _ssrg_signal_planUpdate, set as _ssrg_signal_set, transaction as _ssrg_signal_transaction, read as _ssrg_signal_read, type MutableSignal as MutableSignal, type Signal as Signal, type SignalChange as SignalChange } from "@seseragi/runtime/signal"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

const twice = (value: number) => _ssrg_int_add(value, value)
const add = (left: number) => (right: number) => _ssrg_int_add(left, right)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_signal_make<number>(20), (left: MutableSignal<number>) => _ssrg_effect_flatMap(_ssrg_signal_make<number>(1), (right: MutableSignal<number>) => (() => { const doubled: Signal<number> = _ssrg_signal_map(twice, left); return (() => { const total: Signal<number> = _ssrg_signal_combine(add, doubled, right); return (() => { const setLeft: (argument: MutableSignal<number>) => SignalChange = _ssrg_signal_planSet(21); return (() => { const updateRight: (argument: MutableSignal<number>) => SignalChange = _ssrg_signal_planUpdate((value: number) => _ssrg_int_add(value, 20)); return _ssrg_effect_flatMap(_ssrg_signal_set(10, left), () => _ssrg_effect_flatMap(_ssrg_signal_transaction([setLeft(left), updateRight(right)]), () => _ssrg_effect_flatMap(_ssrg_signal_read(total), (current: number) => _ssrg_console_println("signal: " + _ssrg_show_intShow["show"](current))))); })(); })(); })(); })()))

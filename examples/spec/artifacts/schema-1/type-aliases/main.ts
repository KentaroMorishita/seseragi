import { update as _ssrg_signal_update, map as _ssrg_signal_map, make as _ssrg_signal_make, read as _ssrg_signal_read, type MutableSignal as MutableSignal, type Signal as Signal } from "@seseragi/runtime/signal"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { button as _ssrg_html_button, type Html as Html } from "@seseragi/runtime/html"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { succeed as _ssrg_effect_succeed, flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

type AppError =
  | { readonly tag: "AppFailed" };
const AppFailed: AppError = { tag: "AppFailed" } as const;
const userId = (value: number) => value
const pair = <A,>(value: A) => ({ "left": value, "right": value } as const)
const apply = <A, B,>(transform: (argument: A) => B) => (value: A) => transform(value)
const failure = (value: AppError) => value
const result = (value: { readonly tag: "Left"; readonly value: AppError } | { readonly tag: "Right"; readonly value: number }) => value
const increment = (state: MutableSignal<number>) => _ssrg_signal_update((value: number) => _ssrg_int_add(value, 1), state)
const view = (state: MutableSignal<number>) => (current: number) => _ssrg_html_button(({ "onClick": increment(state), "children": _ssrg_show_intShow["show"](current) } as const))
const content = (state: MutableSignal<number>) => _ssrg_signal_map(view(state), state)
const compact = (_unit: undefined) => _ssrg_effect_succeed(undefined)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_signal_make<number>(41), (state: MutableSignal<number>) => _ssrg_effect_flatMap(increment(state), () => (() => { const rendered: Signal<Html<Effect<{  }, never, undefined>>> = content(state); return _ssrg_effect_flatMap(_ssrg_signal_read(state), (current: number) => (() => { const values: { readonly "left": number; readonly "right": number } = pair(apply(userId)(current)); return _ssrg_console_println("aliases: " + _ssrg_show_intShow["show"]((values)["left"]) + ", signal: " + _ssrg_show_intShow["show"](current)); })()); })()))

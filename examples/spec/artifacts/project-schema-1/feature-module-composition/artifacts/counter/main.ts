import { add as _ssrg_int64_add, subtract as _ssrg_int64_subtract } from "@seseragi/runtime/int64"
import { update as _ssrg_signal_update, make as _ssrg_signal_make, map as _ssrg_signal_map, type MutableSignal as MutableSignal, type Signal as Signal } from "@seseragi/runtime/signal"
import { section as _ssrg_html_section, button as _ssrg_html_button, h2 as _ssrg_html_h2, p as _ssrg_html_p, type Html as Html } from "@seseragi/runtime/html"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap, succeed as _ssrg_effect_succeed, type Effect as Effect } from "@seseragi/runtime/effect"

type CounterAction =
  | { readonly tag: "Increment" }
  | { readonly tag: "Decrement" };
const Increment: CounterAction = { tag: "Increment" } as const;
const Decrement: CounterAction = { tag: "Decrement" } as const;
declare const __ssrg$brand$CounterState: unique symbol;
type CounterState = {
  readonly "count": bigint;
  readonly [__ssrg$brand$CounterState]: true;
};
const update = (action: CounterAction) => (state: CounterState) => (($ssrg_match: CounterAction): CounterState => $ssrg_match.tag === "Increment" ? (({ "count": _ssrg_int64_add((state)["count"], 1n) } as const) as unknown as CounterState) : (({ "count": _ssrg_int64_subtract((state)["count"], 1n) } as const) as unknown as CounterState))(action)
const dispatch = (state: MutableSignal<CounterState>) => (action: CounterAction) => _ssrg_signal_update(update(action), state)
const view = (label: string) => (state: MutableSignal<CounterState>) => (current: CounterState) => _ssrg_html_section(({ "children": [_ssrg_html_button(({ "onClick": dispatch(state)(Decrement), "children": "-1" } as const)), _ssrg_html_button(({ "onClick": dispatch(state)(Increment), "children": "+1" } as const)), _ssrg_html_h2(({ "children": label } as const)), _ssrg_html_p(({ "children": "Count: " + _ssrg_show_intShow["show"]((current)["count"]) } as const))] } as const))
export const create = (label: string) => _ssrg_effect_flatMap(_ssrg_signal_make<CounterState>((({ "count": 0n } as const) as unknown as CounterState)), (state: MutableSignal<CounterState>) => _ssrg_effect_succeed(_ssrg_signal_map(view(label)(state), state)))

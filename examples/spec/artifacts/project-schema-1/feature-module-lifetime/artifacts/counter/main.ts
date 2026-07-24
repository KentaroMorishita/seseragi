import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { update as _ssrg_signal_update, make as _ssrg_signal_make, map as _ssrg_signal_map, type MutableSignal as MutableSignal, type Signal as Signal } from "@seseragi/runtime/signal"
import { section as _ssrg_html_section, button as _ssrg_html_button, h2 as _ssrg_html_h2, p as _ssrg_html_p, type Html as Html } from "@seseragi/runtime/html"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap, succeed as _ssrg_effect_succeed, type Effect as Effect } from "@seseragi/runtime/effect"

type CounterAction =
  | { readonly tag: "Increment" };
const Increment: CounterAction = { tag: "Increment" } as const;
declare const __ssrg$brand$CounterState: unique symbol;
type CounterState = {
  readonly "count": bigint;
  readonly [__ssrg$brand$CounterState]: true;
};
const increment = (state: CounterState) => (({ "count": _ssrg_int64_add((state)["count"], 1n) } as const) as unknown as CounterState)
const dispatch = (state: MutableSignal<CounterState>) => (action: CounterAction) => (($ssrg_match: CounterAction): Effect<{  }, never, undefined> => _ssrg_signal_update(increment, state))(action)
const view = (label: string) => (state: MutableSignal<CounterState>) => (current: CounterState) => _ssrg_html_section(({ "key": label, "children": [_ssrg_html_button(({ "onClick": dispatch(state)(Increment), "children": "+1" } as const)), _ssrg_html_h2(({ "children": label } as const)), _ssrg_html_p(({ "children": "Count: " + _ssrg_show_intShow["show"]((current)["count"]) } as const))] } as const))
export const create = (label: string) => _ssrg_effect_flatMap(_ssrg_signal_make<CounterState>((({ "count": 0n } as const) as unknown as CounterState)), (state: MutableSignal<CounterState>) => _ssrg_effect_succeed(_ssrg_signal_map(view(label)(state), state)))

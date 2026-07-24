import { create as createCounter } from "./counter.js"
import { update as _ssrg_signal_update, constant as _ssrg_signal_constant, make as _ssrg_signal_make, map as _ssrg_signal_map, signalApplicative as _ssrg_signal_applicative, signalFunctor as _ssrg_signal_functor, switchMap as _ssrg_signal_switchMap, type MutableSignal as MutableSignal, type Signal as Signal } from "@seseragi/runtime/signal"
import { section as _ssrg_html_section, button as _ssrg_html_button, p as _ssrg_html_p, main as _ssrg_html_main, type Html as Html } from "@seseragi/runtime/html"
import { flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError, type Effect as Effect } from "@seseragi/runtime/effect"
import { query as _ssrg_dom_query, run as _ssrg_dom_run, defaultOptions as _ssrg_dom_defaultOptions, type DomTarget as DomTarget, type DomError as DomError, type Dom as Dom, type DomRuntimeError as DomRuntimeError, type DomOptions as DomOptions } from "@seseragi/runtime/dom"

type LayoutAction =
  | { readonly tag: "ToggleFirst" }
  | { readonly tag: "SwapChildren" }
  | { readonly tag: "ReplaceFirst" };
const ToggleFirst: LayoutAction = { tag: "ToggleFirst" } as const;
const SwapChildren: LayoutAction = { tag: "SwapChildren" } as const;
const ReplaceFirst: LayoutAction = { tag: "ReplaceFirst" } as const;
declare const __ssrg$brand$LayoutState: unique symbol;
type LayoutState = {
  readonly "showFirst": boolean;
  readonly "reversed": boolean;
  readonly "replacement": boolean;
  readonly [__ssrg$brand$LayoutState]: true;
};
const toggle = (value: boolean) => value ? false : true
const update = (action: LayoutAction) => (state: LayoutState) => (($ssrg_match: LayoutAction): LayoutState => $ssrg_match.tag === "ToggleFirst" ? (({ ...state, "showFirst": toggle((state)["showFirst"]) } as const) as unknown as LayoutState) : $ssrg_match.tag === "SwapChildren" ? (({ ...state, "reversed": toggle((state)["reversed"]) } as const) as unknown as LayoutState) : (({ ...state, "replacement": toggle((state)["replacement"]) } as const) as unknown as LayoutState))(action)
const dispatch = (state: MutableSignal<LayoutState>) => (action: LayoutAction) => _ssrg_signal_update(update(action), state)
const controls = (state: MutableSignal<LayoutState>) => (current: LayoutState) => _ssrg_html_section(({ "children": [_ssrg_html_button(({ "onClick": dispatch(state)(ToggleFirst), "children": "Hide / show first" } as const)), _ssrg_html_button(({ "onClick": dispatch(state)(SwapChildren), "children": "Swap order" } as const)), _ssrg_html_button(({ "onClick": dispatch(state)(ReplaceFirst), "children": "Replace first" } as const)), _ssrg_html_p(({ "children": (current)["showFirst"] ? "First feature is mounted" : "First feature is hidden" } as const))] } as const))
const hidden = (_unit: undefined) => _ssrg_html_p(({ "children": "First feature is outside the tree" } as const))
const selectFirst = (first: Signal<Html<Effect<{  }, never, undefined>>>) => (replacement: Signal<Html<Effect<{  }, never, undefined>>>) => (layout: LayoutState) => (layout)["showFirst"] ? (layout)["replacement"] ? replacement : first : _ssrg_signal_constant(hidden(undefined))
const page = (layout: LayoutState) => (panel: Html<Effect<{  }, never, undefined>>) => (first: Html<Effect<{  }, never, undefined>>) => (second: Html<Effect<{  }, never, undefined>>) => _ssrg_html_main(({ "children": (layout)["reversed"] ? [panel, second, first] : [panel, first, second] } as const))
const perform = (action: Effect<{  }, never, undefined>) => action
export const start = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_signal_make<LayoutState>((({ "showFirst": true, "reversed": false, "replacement": false } as const) as unknown as LayoutState)), (layout: MutableSignal<LayoutState>) => _ssrg_effect_flatMap(createCounter("Counter A"), (first: Signal<Html<Effect<{  }, never, undefined>>>) => _ssrg_effect_flatMap(createCounter("Counter B"), (second: Signal<Html<Effect<{  }, never, undefined>>>) => _ssrg_effect_flatMap(createCounter("Replacement"), (replacement: Signal<Html<Effect<{  }, never, undefined>>>) => (() => { const panel: Signal<Html<Effect<{  }, never, undefined>>> = _ssrg_signal_map(controls(layout), layout); return (() => { const layoutView: Signal<LayoutState> = _ssrg_signal_map((current: LayoutState) => current, layout); return (() => { const content: Signal<Html<Effect<{  }, never, undefined>>> = _ssrg_signal_applicative["apply"](_ssrg_signal_applicative["apply"](_ssrg_signal_applicative["apply"](_ssrg_signal_functor["map"](page)(layoutView))(panel))(_ssrg_signal_switchMap((current: LayoutState) => selectFirst(first)(replacement)(current), layout)))(second); return _ssrg_effect_flatMap(_ssrg_effect_mapError((error: DomError) => "DOM target unavailable", _ssrg_dom_query("#app")), (target: DomTarget) => _ssrg_effect_mapError((error: DomRuntimeError<never>) => "DOM runtime failed", _ssrg_dom_run(_ssrg_dom_defaultOptions(undefined), target, perform, content))); })(); })(); })()))))

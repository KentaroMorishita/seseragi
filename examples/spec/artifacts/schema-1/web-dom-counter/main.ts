import { main as _ssrg_html_main, style as _ssrg_html_style, h1 as _ssrg_html_h1, p as _ssrg_html_p, button as _ssrg_html_button, type Html as Html, type Style as Style } from "@seseragi/runtime/html"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError } from "@seseragi/runtime/effect"
import { make as _ssrg_signal_make, map as _ssrg_signal_map, update as _ssrg_signal_update, type MutableSignal as MutableSignal, type Signal as Signal } from "@seseragi/runtime/signal"
import { query as _ssrg_dom_query, run as _ssrg_dom_run, defaultOptions as _ssrg_dom_defaultOptions, type DomTarget as DomTarget, type DomError as DomError, type Dom as Dom, type DomRuntimeError as DomRuntimeError, type DomOptions as DomOptions } from "@seseragi/runtime/dom"
import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"

type Msg =
  | { readonly tag: "Increment" };
const Increment: Msg = { tag: "Increment" } as const;
const view = (count: bigint) => _ssrg_html_main(({ "style": _ssrg_html_style(({ "backgroundColor": "#eef6f3", "padding": "32px" } as const)), "children": [_ssrg_html_h1(({ "style": _ssrg_html_style(({ "color": "#123b32", "marginTop": "0" } as const)), "children": "Interactive Seseragi" } as const)), _ssrg_html_p(({ "id": "count", "style": _ssrg_html_style(({ "color": "#185f50", "fontSize": "24px" } as const)), "children": "Count: " + _ssrg_show_intShow["show"](count) } as const)), _ssrg_html_button(({ "id": "increment", "onClick": Increment, "style": _ssrg_html_style(({ "backgroundColor": "#8ce6c3", "borderRadius": "999px", "padding": "12px 20px" } as const)), "children": "+1" } as const))] } as const))
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_signal_make(0n), (count: MutableSignal<bigint>) => (() => { const content: Signal<Html<Msg>> = _ssrg_signal_map(view, count); return _ssrg_effect_flatMap(_ssrg_effect_mapError((error: DomError) => "DOM target unavailable", _ssrg_dom_query("#app")), (target: DomTarget) => _ssrg_effect_mapError((error: DomRuntimeError<never>) => "DOM runtime failed", _ssrg_dom_run(_ssrg_dom_defaultOptions(undefined), target, (message: Msg) => _ssrg_signal_update((value: bigint) => _ssrg_int64_add(value, 1n), count), content))); })())

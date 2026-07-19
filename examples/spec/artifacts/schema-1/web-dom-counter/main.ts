import { style as _ssrg_html_style, button as _ssrg_html_button, main as _ssrg_html_main, section as _ssrg_html_section, div as _ssrg_html_div, span as _ssrg_html_span, h1 as _ssrg_html_h1, p as _ssrg_html_p, type Style as Style, type Html as Html } from "@seseragi/runtime/html"
import { update as _ssrg_signal_update, make as _ssrg_signal_make, map as _ssrg_signal_map, type MutableSignal as MutableSignal, type Signal as Signal } from "@seseragi/runtime/signal"
import { flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError, type Effect as Effect } from "@seseragi/runtime/effect"
import { query as _ssrg_dom_query, run as _ssrg_dom_run, defaultOptions as _ssrg_dom_defaultOptions, type DomTarget as DomTarget, type DomError as DomError, type Dom as Dom, type DomRuntimeError as DomRuntimeError, type DomOptions as DomOptions } from "@seseragi/runtime/dom"

type Mode =
  | { readonly tag: "Ready" }
  | { readonly tag: "Focusing" }
  | { readonly tag: "Resting" };
const Ready: Mode = { tag: "Ready" } as const;
const Focusing: Mode = { tag: "Focusing" } as const;
const Resting: Mode = { tag: "Resting" } as const;
type Msg =
  | { readonly tag: "StartFocus" }
  | { readonly tag: "TakeBreak" }
  | { readonly tag: "Reset" };
const StartFocus: Msg = { tag: "StartFocus" } as const;
const TakeBreak: Msg = { tag: "TakeBreak" } as const;
const Reset: Msg = { tag: "Reset" } as const;
const initialMode: Mode = Ready;
const pageStyle: Style = _ssrg_html_style(({ "background": "linear-gradient(135deg, #d1fae5 0%, #f8fafc 48%, #dbeafe 100%)", "minHeight": "100vh", "padding": "32px" } as const));
const cardStyle: Style = _ssrg_html_style(({ "variables": ({ "cardShadow": "0 24px 64px #0f172a24" } as const), "backgroundColor": "#ffffff", "border": "1px solid #ffffffcc", "borderRadius": "28px", "boxShadow": "var(--card-shadow)", "padding": "32px" } as const));
const trackStyle: Style = _ssrg_html_style(({ "backgroundColor": "#e2e8f0", "borderRadius": "999px", "height": "10px", "overflow": "hidden" } as const));
const label = (mode: Mode) => (($ssrg_match: Mode): string => $ssrg_match.tag === "Ready" ? "READY" : $ssrg_match.tag === "Focusing" ? "FOCUS" : "BREAK")(mode)
const title = (mode: Mode) => (($ssrg_match: Mode): string => $ssrg_match.tag === "Ready" ? "Choose your rhythm" : $ssrg_match.tag === "Focusing" ? "Deep work in progress" : "Pause with purpose")(mode)
const guidance = (mode: Mode) => (($ssrg_match: Mode): string => $ssrg_match.tag === "Ready" ? "Pick a mode and let a typed message update the Signal." : $ssrg_match.tag === "Focusing" ? "One task. No tab hopping. You have got this." : "Look away from the screen and take a slow breath.")(mode)
const accent = (mode: Mode) => (($ssrg_match: Mode): string => $ssrg_match.tag === "Ready" ? "#475569" : $ssrg_match.tag === "Focusing" ? "#059669" : "#2563eb")(mode)
const progress = (mode: Mode) => (($ssrg_match: Mode): string => $ssrg_match.tag === "Ready" ? "18%" : $ssrg_match.tag === "Focusing" ? "82%" : "46%")(mode)
const progressStyle = (mode: Mode) => _ssrg_html_style(({ "backgroundColor": accent(mode), "borderRadius": "999px", "height": "100%", "transition": "all 240ms ease", "width": progress(mode) } as const))
const badgeStyle = (mode: Mode) => _ssrg_html_style(({ "backgroundColor": accent(mode), "borderRadius": "999px", "color": "#ffffff", "fontSize": "12px", "fontWeight": "700", "letterSpacing": "0.12em", "padding": "6px 12px" } as const))
const actionStyle = (message: Msg) => (($ssrg_match: Msg): Style => $ssrg_match.tag === "StartFocus" ? _ssrg_html_style(({ "backgroundColor": "#059669", "color": "#ffffff" } as const)) : $ssrg_match.tag === "TakeBreak" ? _ssrg_html_style(({ "backgroundColor": "#2563eb", "color": "#ffffff" } as const)) : _ssrg_html_style(({ "backgroundColor": "#f1f5f9", "color": "#334155" } as const)))(message)
const action = (id: string) => (text: string) => (message: Msg) => _ssrg_html_button(({ "id": id, "onClick": message, "className": "w-full rounded-xl border-0 px-4 py-3 font-semibold transition hover:shadow-lg", "style": actionStyle(message), "children": text } as const))
const view = (mode: Mode) => _ssrg_html_main(({ "style": pageStyle, "children": [_ssrg_html_section(({ "className": "mx-auto max-w-lg", "style": cardStyle, "children": [_ssrg_html_div(({ "className": "mb-6 flex items-center justify-between", "children": [_ssrg_html_span(({ "style": badgeStyle(mode), "children": label(mode) } as const)), _ssrg_html_span(({ "className": "text-sm font-semibold text-slate-500", "children": "SESERAGI FLOW" } as const))] } as const)), _ssrg_html_h1(({ "className": "m-0 text-3xl font-bold text-slate-900 sm:text-4xl", "children": title(mode) } as const)), _ssrg_html_p(({ "className": "mb-6 mt-4 text-lg text-slate-700", "children": guidance(mode) } as const)), _ssrg_html_div(({ "style": trackStyle, "children": _ssrg_html_div(({ "style": progressStyle(mode), "children": "" } as const)) } as const)), _ssrg_html_div(({ "className": "mt-6 grid grid-cols-1 gap-2 sm:grid-cols-3", "children": [action("focus")("Start focus")(StartFocus), action("break")("Take a break")(TakeBreak), action("reset")("Reset the flow")(Reset)] } as const))] } as const))] } as const))
const update = (mode: MutableSignal<Mode>) => (message: Msg) => (($ssrg_match: Msg): Effect<{  }, never, undefined> => $ssrg_match.tag === "StartFocus" ? _ssrg_signal_update((current: Mode) => Focusing, mode) : $ssrg_match.tag === "TakeBreak" ? _ssrg_signal_update((current: Mode) => Resting, mode) : _ssrg_signal_update((current: Mode) => Ready, mode))(message)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_signal_make<Mode>(initialMode), (mode: MutableSignal<Mode>) => (() => { const content: Signal<Html<Msg>> = _ssrg_signal_map(view, mode); return _ssrg_effect_flatMap(_ssrg_effect_mapError((error: DomError) => "DOM target unavailable", _ssrg_dom_query("#app")), (target: DomTarget) => _ssrg_effect_mapError((error: DomRuntimeError<never>) => "DOM runtime failed", _ssrg_dom_run(_ssrg_dom_defaultOptions(undefined), target, update(mode), content))); })())

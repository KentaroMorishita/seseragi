import { style as _ssrg_html_style, button as _ssrg_html_button, main as _ssrg_html_main, section as _ssrg_html_section, div as _ssrg_html_div, span as _ssrg_html_span, h1 as _ssrg_html_h1, p as _ssrg_html_p, type Style as Style, type Html as Html } from "@seseragi/runtime/html"
import { app as _ssrg_dom_app, type Dom as Dom } from "@seseragi/runtime/dom"
import { type Effect as Effect } from "@seseragi/runtime/effect"

type Mode =
  | { readonly tag: "Ready" }
  | { readonly tag: "Focusing" }
  | { readonly tag: "Resting" };
const Ready: Mode = { tag: "Ready" } as const;
const Focusing: Mode = { tag: "Focusing" } as const;
const Resting: Mode = { tag: "Resting" } as const;
type Action =
  | { readonly tag: "StartFocus" }
  | { readonly tag: "TakeBreak" }
  | { readonly tag: "Reset" };
const StartFocus: Action = { tag: "StartFocus" } as const;
const TakeBreak: Action = { tag: "TakeBreak" } as const;
const Reset: Action = { tag: "Reset" } as const;
const initialMode: Mode = Ready;
const pageStyle: Style = _ssrg_html_style(({ "background": "linear-gradient(135deg, #d1fae5 0%, #f8fafc 48%, #dbeafe 100%)", "minHeight": "100vh", "padding": "32px" } as const));
const cardStyle: Style = _ssrg_html_style(({ "variables": ({ "cardShadow": "0 24px 64px #0f172a24" } as const), "backgroundColor": "#ffffff", "border": "1px solid #ffffffcc", "borderRadius": "28px", "boxShadow": "var(--card-shadow)", "padding": "32px" } as const));
const trackStyle: Style = _ssrg_html_style(({ "backgroundColor": "#e2e8f0", "borderRadius": "999px", "height": "10px", "overflow": "hidden" } as const));
const label = (mode: Mode) => (($ssrg_match: Mode): string => $ssrg_match.tag === "Ready" ? "READY" : $ssrg_match.tag === "Focusing" ? "FOCUS" : "BREAK")(mode)
const title = (mode: Mode) => (($ssrg_match: Mode): string => $ssrg_match.tag === "Ready" ? "Choose your rhythm" : $ssrg_match.tag === "Focusing" ? "Deep work in progress" : "Pause with purpose")(mode)
const guidance = (mode: Mode) => (($ssrg_match: Mode): string => $ssrg_match.tag === "Ready" ? "Pick a mode and let a typed action update the Signal." : $ssrg_match.tag === "Focusing" ? "One task. No tab hopping. You have got this." : "Look away from the screen and take a slow breath.")(mode)
const accent = (mode: Mode) => (($ssrg_match: Mode): string => $ssrg_match.tag === "Ready" ? "#475569" : $ssrg_match.tag === "Focusing" ? "#059669" : "#2563eb")(mode)
const progress = (mode: Mode) => (($ssrg_match: Mode): string => $ssrg_match.tag === "Ready" ? "18%" : $ssrg_match.tag === "Focusing" ? "82%" : "46%")(mode)
const progressStyle = (mode: Mode) => _ssrg_html_style(({ "backgroundColor": accent(mode), "borderRadius": "999px", "height": "100%", "transition": "all 240ms ease", "width": progress(mode) } as const))
const badgeStyle = (mode: Mode) => _ssrg_html_style(({ "backgroundColor": accent(mode), "borderRadius": "999px", "color": "#ffffff", "fontSize": "12px", "fontWeight": "700", "letterSpacing": "0.12em", "padding": "6px 12px" } as const))
const actionStyle = (action: Action) => (($ssrg_match: Action): Style => $ssrg_match.tag === "StartFocus" ? _ssrg_html_style(({ "backgroundColor": "#059669", "color": "#ffffff" } as const)) : $ssrg_match.tag === "TakeBreak" ? _ssrg_html_style(({ "backgroundColor": "#2563eb", "color": "#ffffff" } as const)) : _ssrg_html_style(({ "backgroundColor": "#f1f5f9", "color": "#334155" } as const)))(action)
const actionButton = (id: string) => (text: string) => (action: Action) => _ssrg_html_button(({ "id": id, "onClick": action, "className": "w-full rounded-xl border-0 px-4 py-3 font-semibold transition hover:shadow-lg", "style": actionStyle(action), "children": text } as const))
const view = (mode: Mode) => _ssrg_html_main(({ "style": pageStyle, "children": [_ssrg_html_section(({ "className": "mx-auto max-w-lg", "style": cardStyle, "children": [_ssrg_html_div(({ "className": "mb-6 flex items-center justify-between", "children": [_ssrg_html_span(({ "style": badgeStyle(mode), "children": label(mode) } as const)), _ssrg_html_span(({ "className": "text-sm font-semibold text-slate-500", "children": "SESERAGI FLOW" } as const))] } as const)), _ssrg_html_h1(({ "className": "m-0 text-3xl font-bold text-slate-900 sm:text-4xl", "children": title(mode) } as const)), _ssrg_html_p(({ "className": "mb-6 mt-4 text-lg text-slate-700", "children": guidance(mode) } as const)), _ssrg_html_div(({ "style": trackStyle, "children": _ssrg_html_div(({ "style": progressStyle(mode), "children": "" } as const)) } as const)), _ssrg_html_div(({ "className": "mt-6 grid grid-cols-1 gap-2 sm:grid-cols-3", "children": [actionButton("focus")("Start focus")(StartFocus), actionButton("break")("Take a break")(TakeBreak), actionButton("reset")("Reset the flow")(Reset)] } as const))] } as const))] } as const))
const update = (action: Action) => (mode: Mode) => (($ssrg_match: Action): Mode => $ssrg_match.tag === "StartFocus" ? Focusing : $ssrg_match.tag === "TakeBreak" ? Resting : Ready)(action)
export const main = (_unit: undefined) => _ssrg_dom_app(({ "target": "#app", "initial": initialMode, "update": update, "view": view } as const))

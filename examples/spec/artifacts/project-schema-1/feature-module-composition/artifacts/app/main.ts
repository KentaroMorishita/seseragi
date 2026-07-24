import { create as createCounter } from "./counter.js"
import { create as createTodo } from "./todo/feature.js"
import { update as _ssrg_signal_update, make as _ssrg_signal_make, map as _ssrg_signal_map, signalApplicative as _ssrg_signal_applicative, signalFunctor as _ssrg_signal_functor, type MutableSignal as MutableSignal, type Signal as Signal } from "@seseragi/runtime/signal"
import { section as _ssrg_html_section, button as _ssrg_html_button, h1 as _ssrg_html_h1, main as _ssrg_html_main, type Html as Html } from "@seseragi/runtime/html"
import { stringShow as _ssrg_show_stringShow } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError, type Effect as Effect } from "@seseragi/runtime/effect"
import { query as _ssrg_dom_query, run as _ssrg_dom_run, defaultOptions as _ssrg_dom_defaultOptions, type DomTarget as DomTarget, type DomError as DomError, type Dom as Dom, type DomRuntimeError as DomRuntimeError, type DomOptions as DomOptions } from "@seseragi/runtime/dom"

type Theme =
  | { readonly tag: "Light" }
  | { readonly tag: "Dark" };
const Light: Theme = { tag: "Light" } as const;
const Dark: Theme = { tag: "Dark" } as const;
type AppAction =
  | { readonly tag: "ToggleTheme" };
const ToggleTheme: AppAction = { tag: "ToggleTheme" } as const;
declare const __ssrg$brand$AppState: unique symbol;
type AppState = {
  readonly "theme": Theme;
  readonly [__ssrg$brand$AppState]: true;
};
const toggle = (theme: Theme) => (($ssrg_match: Theme): Theme => $ssrg_match.tag === "Light" ? Dark : Light)(theme)
const update = (action: AppAction) => (state: AppState) => (($ssrg_match: AppAction): AppState => (({ "theme": toggle((state)["theme"]) } as const) as unknown as AppState))(action)
const dispatch = (state: MutableSignal<AppState>) => (action: AppAction) => _ssrg_signal_update(update(action), state)
const themeName = (theme: Theme) => (($ssrg_match: Theme): string => $ssrg_match.tag === "Light" ? "Light" : "Dark")(theme)
const appView = (state: MutableSignal<AppState>) => (current: AppState) => _ssrg_html_section(({ "children": [_ssrg_html_button(({ "onClick": dispatch(state)(ToggleTheme), "children": "Toggle app theme" } as const)), _ssrg_html_h1(({ "children": "Feature tree: " + _ssrg_show_stringShow["show"](themeName((current)["theme"])) } as const))] } as const))
const page = (app: Html<Effect<{  }, never, undefined>>) => (first: Html<Effect<{  }, never, undefined>>) => (second: Html<Effect<{  }, never, undefined>>) => (todo: Html<Effect<{  }, never, undefined>>) => _ssrg_html_main(({ "children": [app, first, second, todo] } as const))
const perform = (action: Effect<{  }, never, undefined>) => action
export const start = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_signal_make<AppState>((({ "theme": Light } as const) as unknown as AppState)), (appState: MutableSignal<AppState>) => _ssrg_effect_flatMap(createCounter("Counter A"), (first: Signal<Html<Effect<{  }, never, undefined>>>) => _ssrg_effect_flatMap(createCounter("Counter B"), (second: Signal<Html<Effect<{  }, never, undefined>>>) => _ssrg_effect_flatMap(createTodo(undefined), (todo: Signal<Html<Effect<{  }, never, undefined>>>) => (() => { const app: Signal<Html<Effect<{  }, never, undefined>>> = _ssrg_signal_map(appView(appState), appState); return (() => { const content: Signal<Html<Effect<{  }, never, undefined>>> = _ssrg_signal_applicative["apply"](_ssrg_signal_applicative["apply"](_ssrg_signal_applicative["apply"](_ssrg_signal_functor["map"](page)(app))(first))(second))(todo); return _ssrg_effect_flatMap(_ssrg_effect_mapError((error: DomError) => "DOM target unavailable", _ssrg_dom_query("#app")), (target: DomTarget) => _ssrg_effect_mapError((error: DomRuntimeError<never>) => "DOM runtime failed", _ssrg_dom_run(_ssrg_dom_defaultOptions(undefined), target, perform, content))); })(); })()))))

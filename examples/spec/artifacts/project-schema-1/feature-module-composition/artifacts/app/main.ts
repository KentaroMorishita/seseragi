import { create as createCounter } from "./counter.js"
import { create as createTodo } from "./todo/feature.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { update as _ssrg_signal_update, make as _ssrg_signal_make, map as _ssrg_signal_map, signalApplicative as _ssrg_signal_applicative, signalFunctor as _ssrg_signal_functor, type MutableSignal as MutableSignal, type Signal as Signal } from "@seseragi/runtime/signal"
import { section as _ssrg_html_section, button as _ssrg_html_button, h1 as _ssrg_html_h1, main as _ssrg_html_main, type Html as Html } from "@seseragi/runtime/html"
import { stringShow as _ssrg_show_stringShow } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError, type Effect as Effect } from "@seseragi/runtime/effect"
import { query as _ssrg_dom_query, run as _ssrg_dom_run, defaultOptions as _ssrg_dom_defaultOptions, type DomTarget as DomTarget, type DomError as DomError, type Dom as Dom, type DomRuntimeError as DomRuntimeError, type DomOptions as DomOptions } from "@seseragi/runtime/dom"
const $ssrg$source570 = { file: "project/feature-module-composition/src/app.ssrg", line: 27, column: 38, uri: "seseragi://fixture/feature-module-composition::app", start: 570, end: 588 };
const $ssrg$source695 = { file: "project/feature-module-composition/src/app.ssrg", line: 33, column: 3, uri: "seseragi://fixture/feature-module-composition::app", start: 695, end: 731 };
const $ssrg$source711 = { file: "project/feature-module-composition/src/app.ssrg", line: 33, column: 19, uri: "seseragi://fixture/feature-module-composition::app", start: 711, end: 724 };
const $ssrg$source941 = { file: "project/feature-module-composition/src/app.ssrg", line: 44, column: 3, uri: "seseragi://fixture/feature-module-composition::app", start: 941, end: 1163 };
const $ssrg$source978 = { file: "project/feature-module-composition/src/app.ssrg", line: 46, column: 7, uri: "seseragi://fixture/feature-module-composition::app", start: 978, end: 1081 };
const $ssrg$source1009 = { file: "project/feature-module-composition/src/app.ssrg", line: 47, column: 18, uri: "seseragi://fixture/feature-module-composition::app", start: 1009, end: 1035 };
const $ssrg$source1089 = { file: "project/feature-module-composition/src/app.ssrg", line: 50, column: 7, uri: "seseragi://fixture/feature-module-composition::app", start: 1089, end: 1153 };
const $ssrg$source1126 = { file: "project/feature-module-composition/src/app.ssrg", line: 50, column: 44, uri: "seseragi://fixture/feature-module-composition::app", start: 1126, end: 1149 };
const $ssrg$source1398 = { file: "project/feature-module-composition/src/app.ssrg", line: 59, column: 3, uri: "seseragi://fixture/feature-module-composition::app", start: 1398, end: 1448 };
const $ssrg$source1614 = { file: "project/feature-module-composition/src/app.ssrg", line: 67, column: 17, uri: "seseragi://fixture/feature-module-composition::app", start: 1614, end: 1654 };
const $ssrg$source1668 = { file: "project/feature-module-composition/src/app.ssrg", line: 68, column: 14, uri: "seseragi://fixture/feature-module-composition::app", start: 1668, end: 1693 };
const $ssrg$source1708 = { file: "project/feature-module-composition/src/app.ssrg", line: 69, column: 15, uri: "seseragi://fixture/feature-module-composition::app", start: 1708, end: 1733 };
const $ssrg$source1746 = { file: "project/feature-module-composition/src/app.ssrg", line: 70, column: 13, uri: "seseragi://fixture/feature-module-composition::app", start: 1746, end: 1759 };
const $ssrg$source1775 = { file: "project/feature-module-composition/src/app.ssrg", line: 72, column: 15, uri: "seseragi://fixture/feature-module-composition::app", start: 1775, end: 1814 };
const $ssrg$source1788 = { file: "project/feature-module-composition/src/app.ssrg", line: 72, column: 28, uri: "seseragi://fixture/feature-module-composition::app", start: 1788, end: 1804 };
const $ssrg$source1833 = { file: "project/feature-module-composition/src/app.ssrg", line: 73, column: 19, uri: "seseragi://fixture/feature-module-composition::app", start: 1833, end: 1845 };
const $ssrg$source1897 = { file: "project/feature-module-composition/src/app.ssrg", line: 76, column: 7, uri: "seseragi://fixture/feature-module-composition::app", start: 1897, end: 1913 };
const $ssrg$source1990 = { file: "project/feature-module-composition/src/app.ssrg", line: 79, column: 5, uri: "seseragi://fixture/feature-module-composition::app", start: 1990, end: 2044 };
const $ssrg$source1999 = { file: "project/feature-module-composition/src/app.ssrg", line: 79, column: 14, uri: "seseragi://fixture/feature-module-composition::app", start: 1999, end: 2020 };
$ssrg$assertUnicodeVersion("17.0.0")

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
const update = (action: AppAction) => (state: AppState) => (($ssrg_match: AppAction): AppState => (({ "theme": $ssrg$atSource($ssrg$source570, () => toggle((state)["theme"]), false) } as const) as unknown as AppState))(action)
const dispatch = (state: MutableSignal<AppState>) => (action: AppAction) => $ssrg$atSource($ssrg$source695, () => _ssrg_signal_update($ssrg$atSource($ssrg$source711, () => update(action), false), state), true)
const themeName = (theme: Theme) => (($ssrg_match: Theme): string => $ssrg_match.tag === "Light" ? "Light" : "Dark")(theme)
const appView = (state: MutableSignal<AppState>) => (current: AppState) => $ssrg$atSource($ssrg$source941, () => _ssrg_html_section((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "children": [$ssrg$atSource($ssrg$source978, () => _ssrg_html_button((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "onClick": $ssrg$atSource($ssrg$source1009, () => dispatch(state)(ToggleTheme), true), "children": "Toggle app theme" } as const))), false), $ssrg$atSource($ssrg$source1089, () => _ssrg_html_h1((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "children": "Feature tree: " + _ssrg_show_stringShow["show"]($ssrg$atSource($ssrg$source1126, () => themeName((current)["theme"]), false)) } as const))), false)] } as const))), false)
const page = (app: Html<Effect<{  }, never, undefined>>) => (first: Html<Effect<{  }, never, undefined>>) => (second: Html<Effect<{  }, never, undefined>>) => (todo: Html<Effect<{  }, never, undefined>>) => $ssrg$atSource($ssrg$source1398, () => _ssrg_html_main((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "children": [app, first, second, todo] } as const))), false)
const perform = (action: Effect<{  }, never, undefined>) => action
export const start = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1614, () => _ssrg_signal_make<AppState>((({ "theme": Light } as const) as unknown as AppState)), true), (appState: MutableSignal<AppState>) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1668, () => createCounter("Counter A"), true), (first: Signal<Html<Effect<{  }, never, undefined>>>) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1708, () => createCounter("Counter B"), true), (second: Signal<Html<Effect<{  }, never, undefined>>>) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1746, () => createTodo(undefined), true), (todo: Signal<Html<Effect<{  }, never, undefined>>>) => (() => { const app: Signal<Html<Effect<{  }, never, undefined>>> = $ssrg$atSource($ssrg$source1775, () => _ssrg_signal_map($ssrg$atSource($ssrg$source1788, () => appView(appState), false), appState), false); return (() => { const content: Signal<Html<Effect<{  }, never, undefined>>> = $ssrg$atSource($ssrg$source1833, () => ((_ssrg_signal_applicative["apply"]($ssrg$atSource($ssrg$source1833, () => ((_ssrg_signal_applicative["apply"]($ssrg$atSource($ssrg$source1833, () => ((_ssrg_signal_applicative["apply"]($ssrg$atSource($ssrg$source1833, () => ((_ssrg_signal_functor["map"](page)(app)) as Signal<(argument: Html<Effect<{  }, never, undefined>>) => (argument: Html<Effect<{  }, never, undefined>>) => (argument: Html<Effect<{  }, never, undefined>>) => Html<Effect<{  }, never, undefined>>>), false))(first)) as Signal<(argument: Html<Effect<{  }, never, undefined>>) => (argument: Html<Effect<{  }, never, undefined>>) => Html<Effect<{  }, never, undefined>>>), false))(second)) as Signal<(argument: Html<Effect<{  }, never, undefined>>) => Html<Effect<{  }, never, undefined>>>), false))(todo)) as Signal<Html<Effect<{  }, never, undefined>>>), false); return _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1897, () => _ssrg_effect_mapError((error: DomError) => "DOM target unavailable", $ssrg$atSource($ssrg$source1897, () => _ssrg_dom_query("#app"), true)), true), (target: DomTarget) => $ssrg$atSource($ssrg$source1990, () => _ssrg_effect_mapError((error: DomRuntimeError<never>) => "DOM runtime failed", $ssrg$atSource($ssrg$source1990, () => _ssrg_dom_run($ssrg$atSource($ssrg$source1999, () => _ssrg_dom_defaultOptions(undefined), false), target, perform, content), true)), true)); })(); })()))))

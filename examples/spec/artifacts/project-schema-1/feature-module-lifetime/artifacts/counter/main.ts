import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { update as _ssrg_signal_update, make as _ssrg_signal_make, map as _ssrg_signal_map, type MutableSignal as MutableSignal, type Signal as Signal } from "@seseragi/runtime/signal"
import { section as _ssrg_html_section, button as _ssrg_html_button, h2 as _ssrg_html_h2, p as _ssrg_html_p, type Html as Html } from "@seseragi/runtime/html"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap, succeed as _ssrg_effect_succeed, type Effect as Effect } from "@seseragi/runtime/effect"
const $ssrg$source277 = { file: "project/feature-module-lifetime/src/counter.ssrg", line: 13, column: 25, uri: "seseragi://fixture/feature-module-lifetime::counter", start: 277, end: 292 };
const $ssrg$source435 = { file: "project/feature-module-lifetime/src/counter.ssrg", line: 19, column: 18, uri: "seseragi://fixture/feature-module-lifetime::counter", start: 435, end: 465 };
const $ssrg$source604 = { file: "project/feature-module-lifetime/src/counter.ssrg", line: 26, column: 3, uri: "seseragi://fixture/feature-module-lifetime::counter", start: 604, end: 821 };
const $ssrg$source657 = { file: "project/feature-module-lifetime/src/counter.ssrg", line: 29, column: 7, uri: "seseragi://fixture/feature-module-lifetime::counter", start: 657, end: 722 };
const $ssrg$source680 = { file: "project/feature-module-lifetime/src/counter.ssrg", line: 29, column: 30, uri: "seseragi://fixture/feature-module-lifetime::counter", start: 680, end: 704 };
const $ssrg$source730 = { file: "project/feature-module-lifetime/src/counter.ssrg", line: 30, column: 7, uri: "seseragi://fixture/feature-module-lifetime::counter", start: 730, end: 757 };
const $ssrg$source765 = { file: "project/feature-module-lifetime/src/counter.ssrg", line: 31, column: 7, uri: "seseragi://fixture/feature-module-lifetime::counter", start: 765, end: 811 };
const $ssrg$source940 = { file: "project/feature-module-lifetime/src/counter.ssrg", line: 39, column: 14, uri: "seseragi://fixture/feature-module-lifetime::counter", start: 940, end: 980 };
const $ssrg$source985 = { file: "project/feature-module-lifetime/src/counter.ssrg", line: 40, column: 5, uri: "seseragi://fixture/feature-module-lifetime::counter", start: 985, end: 1031 };
const $ssrg$source995 = { file: "project/feature-module-lifetime/src/counter.ssrg", line: 40, column: 15, uri: "seseragi://fixture/feature-module-lifetime::counter", start: 995, end: 1031 };
const $ssrg$source1008 = { file: "project/feature-module-lifetime/src/counter.ssrg", line: 40, column: 28, uri: "seseragi://fixture/feature-module-lifetime::counter", start: 1008, end: 1024 };
$ssrg$assertUnicodeVersion("17.0.0")

type CounterAction =
  | { readonly tag: "Increment" };
const Increment: CounterAction = { tag: "Increment" } as const;
declare const __ssrg$brand$CounterState: unique symbol;
type CounterState = {
  readonly "count": number;
  readonly [__ssrg$brand$CounterState]: true;
};
const increment = (state: CounterState) => (({ "count": _ssrg_int_add((state)["count"], 1, $ssrg$source277) } as const) as unknown as CounterState)
const dispatch = (state: MutableSignal<CounterState>) => (action: CounterAction) => (($ssrg_match: CounterAction): Effect<{  }, never, undefined> => $ssrg$atSource($ssrg$source435, () => _ssrg_signal_update(increment, state), true))(action)
const view = (label: string) => (state: MutableSignal<CounterState>) => (current: CounterState) => $ssrg$atSource($ssrg$source604, () => _ssrg_html_section((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "key": label, "children": [$ssrg$atSource($ssrg$source657, () => _ssrg_html_button((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "onClick": $ssrg$atSource($ssrg$source680, () => dispatch(state)(Increment), true), "children": "+1" } as const))), false), $ssrg$atSource($ssrg$source730, () => _ssrg_html_h2((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "children": label } as const))), false), $ssrg$atSource($ssrg$source765, () => _ssrg_html_p((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "children": "Count: " + _ssrg_show_intShow["show"]((current)["count"]) } as const))), false)] } as const))), false)
export const create = (label: string) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source940, () => _ssrg_signal_make<CounterState>((({ "count": 0 } as const) as unknown as CounterState)), true), (state: MutableSignal<CounterState>) => $ssrg$atSource($ssrg$source985, () => _ssrg_effect_succeed($ssrg$atSource($ssrg$source995, () => _ssrg_signal_map($ssrg$atSource($ssrg$source1008, () => view(label)(state), false), state), false)), true))

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add, subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { update as _ssrg_signal_update, make as _ssrg_signal_make, map as _ssrg_signal_map, type MutableSignal as MutableSignal, type Signal as Signal } from "@seseragi/runtime/signal"
import { section as _ssrg_html_section, button as _ssrg_html_button, h2 as _ssrg_html_h2, p as _ssrg_html_p, type Html as Html } from "@seseragi/runtime/html"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap, succeed as _ssrg_effect_succeed, type Effect as Effect } from "@seseragi/runtime/effect"
const $ssrg$source345 = { file: "project/feature-module-composition/src/counter.ssrg", line: 15, column: 40, uri: "seseragi://fixture/feature-module-composition::counter", start: 345, end: 360 };
const $ssrg$source402 = { file: "project/feature-module-composition/src/counter.ssrg", line: 16, column: 40, uri: "seseragi://fixture/feature-module-composition::counter", start: 402, end: 417 };
const $ssrg$source532 = { file: "project/feature-module-composition/src/counter.ssrg", line: 22, column: 3, uri: "seseragi://fixture/feature-module-composition::counter", start: 532, end: 568 };
const $ssrg$source548 = { file: "project/feature-module-composition/src/counter.ssrg", line: 22, column: 19, uri: "seseragi://fixture/feature-module-composition::counter", start: 548, end: 561 };
const $ssrg$source703 = { file: "project/feature-module-composition/src/counter.ssrg", line: 28, column: 3, uri: "seseragi://fixture/feature-module-composition::counter", start: 703, end: 977 };
const $ssrg$source740 = { file: "project/feature-module-composition/src/counter.ssrg", line: 30, column: 7, uri: "seseragi://fixture/feature-module-composition::counter", start: 740, end: 805 };
const $ssrg$source763 = { file: "project/feature-module-composition/src/counter.ssrg", line: 30, column: 30, uri: "seseragi://fixture/feature-module-composition::counter", start: 763, end: 787 };
const $ssrg$source813 = { file: "project/feature-module-composition/src/counter.ssrg", line: 31, column: 7, uri: "seseragi://fixture/feature-module-composition::counter", start: 813, end: 878 };
const $ssrg$source836 = { file: "project/feature-module-composition/src/counter.ssrg", line: 31, column: 30, uri: "seseragi://fixture/feature-module-composition::counter", start: 836, end: 860 };
const $ssrg$source886 = { file: "project/feature-module-composition/src/counter.ssrg", line: 32, column: 7, uri: "seseragi://fixture/feature-module-composition::counter", start: 886, end: 913 };
const $ssrg$source921 = { file: "project/feature-module-composition/src/counter.ssrg", line: 33, column: 7, uri: "seseragi://fixture/feature-module-composition::counter", start: 921, end: 967 };
const $ssrg$source1096 = { file: "project/feature-module-composition/src/counter.ssrg", line: 41, column: 14, uri: "seseragi://fixture/feature-module-composition::counter", start: 1096, end: 1136 };
const $ssrg$source1141 = { file: "project/feature-module-composition/src/counter.ssrg", line: 42, column: 5, uri: "seseragi://fixture/feature-module-composition::counter", start: 1141, end: 1187 };
const $ssrg$source1151 = { file: "project/feature-module-composition/src/counter.ssrg", line: 42, column: 15, uri: "seseragi://fixture/feature-module-composition::counter", start: 1151, end: 1187 };
const $ssrg$source1164 = { file: "project/feature-module-composition/src/counter.ssrg", line: 42, column: 28, uri: "seseragi://fixture/feature-module-composition::counter", start: 1164, end: 1180 };
$ssrg$assertUnicodeVersion("17.0.0")

type CounterAction =
  | { readonly tag: "Increment" }
  | { readonly tag: "Decrement" };
const Increment: CounterAction = { tag: "Increment" } as const;
const Decrement: CounterAction = { tag: "Decrement" } as const;
declare const __ssrg$brand$CounterState: unique symbol;
type CounterState = {
  readonly "count": number;
  readonly [__ssrg$brand$CounterState]: true;
};
const update = (action: CounterAction) => (state: CounterState) => (($ssrg_match: CounterAction): CounterState => $ssrg_match.tag === "Increment" ? (({ "count": _ssrg_int_add((state)["count"], 1, $ssrg$source345) } as const) as unknown as CounterState) : (({ "count": _ssrg_int_subtract((state)["count"], 1, $ssrg$source402) } as const) as unknown as CounterState))(action)
const dispatch = (state: MutableSignal<CounterState>) => (action: CounterAction) => $ssrg$atSource($ssrg$source532, () => _ssrg_signal_update($ssrg$atSource($ssrg$source548, () => update(action), false), state), true)
const view = (label: string) => (state: MutableSignal<CounterState>) => (current: CounterState) => $ssrg$atSource($ssrg$source703, () => _ssrg_html_section((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "children": [$ssrg$atSource($ssrg$source740, () => _ssrg_html_button((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "onClick": $ssrg$atSource($ssrg$source763, () => dispatch(state)(Decrement), true), "children": "-1" } as const))), false), $ssrg$atSource($ssrg$source813, () => _ssrg_html_button((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "onClick": $ssrg$atSource($ssrg$source836, () => dispatch(state)(Increment), true), "children": "+1" } as const))), false), $ssrg$atSource($ssrg$source886, () => _ssrg_html_h2((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "children": label } as const))), false), $ssrg$atSource($ssrg$source921, () => _ssrg_html_p((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "children": "Count: " + _ssrg_show_intShow["show"]((current)["count"]) } as const))), false)] } as const))), false)
export const create = (label: string) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1096, () => _ssrg_signal_make<CounterState>((({ "count": 0 } as const) as unknown as CounterState)), true), (state: MutableSignal<CounterState>) => $ssrg$atSource($ssrg$source1141, () => _ssrg_effect_succeed($ssrg$atSource($ssrg$source1151, () => _ssrg_signal_map($ssrg$atSource($ssrg$source1164, () => view(label)(state), false), state), false)), true))

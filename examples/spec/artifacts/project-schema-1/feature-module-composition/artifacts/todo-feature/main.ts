import { view as todoForm } from "./form.js"
import { view as todoList } from "./list.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { stringEq as _ssrg_string_eq_dictionary } from "@seseragi/runtime/equality"
import { append as _ssrg_array_append, arrayReducible as _ssrg_array_reducible } from "@seseragi/runtime/array"
import { update as _ssrg_signal_update, make as _ssrg_signal_make, map as _ssrg_signal_map, type MutableSignal as MutableSignal, type Signal as Signal } from "@seseragi/runtime/signal"
import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"
import { section as _ssrg_html_section, h2 as _ssrg_html_h2, type InputEvent as InputEvent, type Html as Html } from "@seseragi/runtime/html"
import { flatMap as _ssrg_effect_flatMap, succeed as _ssrg_effect_succeed, type Effect as Effect } from "@seseragi/runtime/effect"
const $ssrg$source549 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 22, column: 10, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 549, end: 566 };
const $ssrg$source641 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 26, column: 16, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 641, end: 680 };
const $ssrg$source844 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 34, column: 3, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 844, end: 880 };
const $ssrg$source860 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 34, column: 19, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 860, end: 873 };
const $ssrg$source938 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 37, column: 3, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 938, end: 962 };
const $ssrg$source1082 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 43, column: 5, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1082, end: 1213 };
const $ssrg$source1145 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 45, column: 35, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1145, end: 1179 };
const $ssrg$source1161 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 45, column: 51, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1161, end: 1178 };
const $ssrg$source1188 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 46, column: 8, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1188, end: 1212 };
const $ssrg$source1331 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 52, column: 3, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1331, end: 1390 };
const $ssrg$source1341 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 52, column: 13, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1341, end: 1364 };
const $ssrg$source1367 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 52, column: 39, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1367, end: 1389 };
const $ssrg$source1500 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 57, column: 3, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1500, end: 1642 };
const $ssrg$source1537 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 59, column: 7, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1537, end: 1559 };
const $ssrg$source1567 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 60, column: 7, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1567, end: 1589 };
const $ssrg$source1597 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 61, column: 7, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1597, end: 1632 };
const $ssrg$source1747 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 69, column: 14, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1747, end: 1796 };
const $ssrg$source1801 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 70, column: 5, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1801, end: 1841 };
const $ssrg$source1811 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 70, column: 15, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1811, end: 1841 };
const $ssrg$source1824 = { file: "project/feature-module-composition/src/todo/feature.ssrg", line: 70, column: 28, uri: "seseragi://fixture/feature-module-composition::todo/feature", start: 1824, end: 1834 };
$ssrg$assertUnicodeVersion("17.0.0")

type TodoAction =
  | { readonly tag: "DraftChanged"; readonly value: string }
  | { readonly tag: "Submitted" }
  | { readonly tag: "Cleared" };
const DraftChanged = (value: string): TodoAction => ({ tag: "DraftChanged", value } as const);
const Submitted: TodoAction = { tag: "Submitted" } as const;
const Cleared: TodoAction = { tag: "Cleared" } as const;
declare const __ssrg$brand$TodoState: unique symbol;
type TodoState = {
  readonly "draft": string;
  readonly "items": ReadonlyArray<string>;
  readonly [__ssrg$brand$TodoState]: true;
};
const update = (action: TodoAction) => (state: TodoState) => (($ssrg_match: TodoAction): TodoState => $ssrg_match.tag === "DraftChanged" ? ((value: string): TodoState => (({ ...state, "draft": value } as const) as unknown as TodoState))($ssrg_match.value) : $ssrg_match.tag === "Submitted" ? $ssrg$atSource($ssrg$source549, () => _ssrg_string_eq_dictionary["eq"]((state)["draft"])(""), false) ? state : (({ "draft": "", "items": $ssrg$atSource($ssrg$source641, () => _ssrg_array_append<string>([(state)["draft"]], (state)["items"]), false) } as const) as unknown as TodoState) : (({ ...state, "items": [] as ReadonlyArray<string> } as const) as unknown as TodoState))(action)
const dispatch = (state: MutableSignal<TodoState>) => (action: TodoAction) => $ssrg$atSource($ssrg$source844, () => _ssrg_signal_update($ssrg$atSource($ssrg$source860, () => update(action), false), state), true)
const draftAction = (event: InputEvent) => $ssrg$atSource($ssrg$source938, () => DraftChanged((event)["value"]), false)
const formView = (state: MutableSignal<TodoState>) => (current: TodoState) => $ssrg$atSource($ssrg$source1082, () => todoForm((current)["draft"])((event: InputEvent) => $ssrg$atSource($ssrg$source1145, () => dispatch(state)($ssrg$atSource($ssrg$source1161, () => draftAction(event), false)), true))($ssrg$atSource($ssrg$source1188, () => dispatch(state)(Submitted), true)), false)
const listView = (state: MutableSignal<TodoState>) => (current: TodoState) => $ssrg$atSource($ssrg$source1331, () => todoList($ssrg$atSource($ssrg$source1341, () => _ssrg_collection_join(_ssrg_array_reducible, ", ", (current)["items"]), false))($ssrg$atSource($ssrg$source1367, () => dispatch(state)(Cleared), true)), false)
const view = (state: MutableSignal<TodoState>) => (current: TodoState) => $ssrg$atSource($ssrg$source1500, () => _ssrg_html_section((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "children": [$ssrg$atSource($ssrg$source1537, () => formView(state)(current), false), $ssrg$atSource($ssrg$source1567, () => listView(state)(current), false), $ssrg$atSource($ssrg$source1597, () => _ssrg_html_h2((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "children": "Shared Todo" } as const))), false)] } as const))), false)
export const create = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1747, () => _ssrg_signal_make<TodoState>((({ "draft": "", "items": [] as ReadonlyArray<string> } as const) as unknown as TodoState)), true), (state: MutableSignal<TodoState>) => $ssrg$atSource($ssrg$source1801, () => _ssrg_effect_succeed($ssrg$atSource($ssrg$source1811, () => _ssrg_signal_map($ssrg$atSource($ssrg$source1824, () => view(state), false), state), false)), true))

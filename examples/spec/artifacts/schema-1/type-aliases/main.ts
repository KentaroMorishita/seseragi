import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { update as _ssrg_signal_update, map as _ssrg_signal_map, make as _ssrg_signal_make, read as _ssrg_signal_read, type MutableSignal as MutableSignal, type Signal as Signal } from "@seseragi/runtime/signal"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { button as _ssrg_html_button, type Html as Html } from "@seseragi/runtime/html"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { succeed as _ssrg_effect_succeed, flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println, type Console as Console } from "@seseragi/runtime/console"
const $ssrg$source614 = { file: "main.ssrg", line: 25, column: 3, uri: "seseragi://artifact/type-aliases", start: 614, end: 629 };
const $ssrg$source873 = { file: "main.ssrg", line: 34, column: 3, uri: "seseragi://artifact/type-aliases", start: 873, end: 920 };
const $ssrg$source904 = { file: "main.ssrg", line: 34, column: 34, uri: "seseragi://artifact/type-aliases", start: 904, end: 913 };
const $ssrg$source996 = { file: "main.ssrg", line: 39, column: 3, uri: "seseragi://artifact/type-aliases", start: 996, end: 1070 };
const $ssrg$source1023 = { file: "main.ssrg", line: 40, column: 14, uri: "seseragi://artifact/type-aliases", start: 1023, end: 1038 };
const $ssrg$source1135 = { file: "main.ssrg", line: 45, column: 3, uri: "seseragi://artifact/type-aliases", start: 1135, end: 1165 };
const $ssrg$source1148 = { file: "main.ssrg", line: 45, column: 16, uri: "seseragi://artifact/type-aliases", start: 1148, end: 1158 };
const $ssrg$source1187 = { file: "main.ssrg", line: 47, column: 21, uri: "seseragi://artifact/type-aliases", start: 1187, end: 1197 };
const $ssrg$source1280 = { file: "main.ssrg", line: 53, column: 14, uri: "seseragi://artifact/type-aliases", start: 1280, end: 1295 };
const $ssrg$source1300 = { file: "main.ssrg", line: 54, column: 5, uri: "seseragi://artifact/type-aliases", start: 1300, end: 1315 };
const $ssrg$source1335 = { file: "main.ssrg", line: 55, column: 20, uri: "seseragi://artifact/type-aliases", start: 1335, end: 1348 };
const $ssrg$source1364 = { file: "main.ssrg", line: 56, column: 16, uri: "seseragi://artifact/type-aliases", start: 1364, end: 1370 };
const $ssrg$source1388 = { file: "main.ssrg", line: 57, column: 18, uri: "seseragi://artifact/type-aliases", start: 1388, end: 1415 };
const $ssrg$source1395 = { file: "main.ssrg", line: 57, column: 25, uri: "seseragi://artifact/type-aliases", start: 1395, end: 1415 };
const $ssrg$source1420 = { file: "main.ssrg", line: 58, column: 5, uri: "seseragi://artifact/type-aliases", start: 1420, end: 1475 };
$ssrg$assertUnicodeVersion("17.0.0")

type AppError =
  | { readonly tag: "AppFailed" };
const AppFailed: AppError = { tag: "AppFailed" } as const;
const userId = (value: number) => value
const pair = <A,>(value: A) => (<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "left": value, "right": value } as const))
const apply = <A, B,>(transform: (argument: A) => B) => (value: A) => $ssrg$atSource($ssrg$source614, () => transform(value), false)
const app = <A,>(operation: Effect<{ readonly "console": Console }, AppError, A>) => operation
const failure = (value: AppError) => value
const result = (value: { readonly tag: "Left"; readonly value: AppError } | { readonly tag: "Right"; readonly value: number }) => value
const increment = (state: MutableSignal<number>) => $ssrg$atSource($ssrg$source873, () => _ssrg_signal_update((value: number) => _ssrg_int_add(value, 1, $ssrg$source904), state), true)
const view = (state: MutableSignal<number>) => (current: number) => $ssrg$atSource($ssrg$source996, () => _ssrg_html_button((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "onClick": $ssrg$atSource($ssrg$source1023, () => increment(state), true), "children": _ssrg_show_intShow["show"](current) } as const))), false)
const content = (state: MutableSignal<number>) => $ssrg$atSource($ssrg$source1135, () => _ssrg_signal_map($ssrg$atSource($ssrg$source1148, () => view(state), false), state), false)
const compact = (_unit: undefined) => $ssrg$atSource($ssrg$source1187, () => _ssrg_effect_succeed(undefined), true)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1280, () => _ssrg_signal_make<number>(41), true), (state: MutableSignal<number>) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1300, () => increment(state), true), () => (() => { const rendered: Signal<Html<Effect<{  }, never, undefined>>> = $ssrg$atSource($ssrg$source1335, () => content(state), false); return _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1364, () => _ssrg_signal_read(state), true), (current: number) => (() => { const values: { readonly "left": number; readonly "right": number } = $ssrg$atSource($ssrg$source1388, () => pair<number>($ssrg$atSource($ssrg$source1395, () => apply(userId)(current), false)), false); return $ssrg$atSource($ssrg$source1420, () => _ssrg_console_println("aliases: " + _ssrg_show_intShow["show"]((values)["left"]) + ", signal: " + _ssrg_show_intShow["show"](current)), true); })()); })()))

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intShow as _ssrg_show_intShow, intDebug as _ssrg_debug_intDebug, floatShow as _ssrg_show_floatShow, boolDebug as _ssrg_debug_boolDebug, arrayDebug as _ssrg_debug_arrayDebug } from "@seseragi/runtime/show"
import { subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source201 = { file: "main.ssrg", line: 8, column: 25, uri: "seseragi://artifact/unary-operators", start: 201, end: 203 };
const $ssrg$source293 = { file: "main.ssrg", line: 11, column: 24, uri: "seseragi://artifact/unary-operators", start: 293, end: 310 };
const $ssrg$source342 = { file: "main.ssrg", line: 13, column: 31, uri: "seseragi://artifact/unary-operators", start: 342, end: 344 };
const $ssrg$source346 = { file: "main.ssrg", line: 13, column: 35, uri: "seseragi://artifact/unary-operators", start: 346, end: 348 };
const $ssrg$source350 = { file: "main.ssrg", line: 13, column: 39, uri: "seseragi://artifact/unary-operators", start: 350, end: 352 };
const $ssrg$source491 = { file: "main.ssrg", line: 16, column: 38, uri: "seseragi://artifact/unary-operators", start: 491, end: 493 };
const $ssrg$source573 = { file: "main.ssrg", line: 17, column: 65, uri: "seseragi://artifact/unary-operators", start: 573, end: 575 };
const $ssrg$source651 = { file: "main.ssrg", line: 20, column: 15, uri: "seseragi://artifact/unary-operators", start: 651, end: 653 };
const $ssrg$source743 = { file: "main.ssrg", line: 25, column: 38, uri: "seseragi://artifact/unary-operators", start: 743, end: 749 };
const $ssrg$source954 = { file: "main.ssrg", line: 34, column: 36, uri: "seseragi://artifact/unary-operators", start: 954, end: 964 };
const $ssrg$source1065 = { file: "main.ssrg", line: 41, column: 3, uri: "seseragi://artifact/unary-operators", start: 1065, end: 1077 };
const $ssrg$source1111 = { file: "main.ssrg", line: 45, column: 5, uri: "seseragi://artifact/unary-operators", start: 1111, end: 1126 };
const $ssrg$source1131 = { file: "main.ssrg", line: 46, column: 5, uri: "seseragi://artifact/unary-operators", start: 1131, end: 1133 };
const $ssrg$source1182 = { file: "main.ssrg", line: 50, column: 5, uri: "seseragi://artifact/unary-operators", start: 1182, end: 1184 };
const $ssrg$source1216 = { file: "main.ssrg", line: 53, column: 5, uri: "seseragi://artifact/unary-operators", start: 1216, end: 1218 };
const $ssrg$source1252 = { file: "main.ssrg", line: 56, column: 5, uri: "seseragi://artifact/unary-operators", start: 1252, end: 1268 };
const $ssrg$source1288 = { file: "main.ssrg", line: 59, column: 5, uri: "seseragi://artifact/unary-operators", start: 1288, end: 1306 };
const $ssrg$source1326 = { file: "main.ssrg", line: 62, column: 5, uri: "seseragi://artifact/unary-operators", start: 1326, end: 1346 };
const $ssrg$source1335 = { file: "main.ssrg", line: 62, column: 14, uri: "seseragi://artifact/unary-operators", start: 1335, end: 1345 };
const $ssrg$source1342 = { file: "main.ssrg", line: 62, column: 21, uri: "seseragi://artifact/unary-operators", start: 1342, end: 1344 };
const $ssrg$source1351 = { file: "main.ssrg", line: 63, column: 5, uri: "seseragi://artifact/unary-operators", start: 1351, end: 1375 };
const $ssrg$source1360 = { file: "main.ssrg", line: 63, column: 14, uri: "seseragi://artifact/unary-operators", start: 1360, end: 1374 };
const $ssrg$source1367 = { file: "main.ssrg", line: 63, column: 21, uri: "seseragi://artifact/unary-operators", start: 1367, end: 1369 };
const $ssrg$source1371 = { file: "main.ssrg", line: 63, column: 25, uri: "seseragi://artifact/unary-operators", start: 1371, end: 1373 };
const $ssrg$source1380 = { file: "main.ssrg", line: 64, column: 5, uri: "seseragi://artifact/unary-operators", start: 1380, end: 1382 };
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Snapshot: unique symbol;
export type Snapshot = {
  readonly "negative": number;
  readonly "negativeZero": number;
  readonly "inverted": boolean;
  readonly [__ssrg$brand$Snapshot]: true;
};
export const __ssrg$instance$Render$0 = { "render": (value: number) => $ssrg$atSource($ssrg$source954, () => ((_ssrg_show_intShow["show"](value)) as string), false) } as const;
export const negateInt = (value: number) => _ssrg_int_subtract(0, value, $ssrg$source743)
export const negateFloat = (value: number) => -(value)
export const invert = (value: boolean) => !(value)
const identity = <A,>(value: A) => value
const __ssrg$operator$3c7e3e = (left: number) => (right: number) => _ssrg_int_subtract(left, right, $ssrg$source1065)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1111, () => _ssrg_console_println("first"), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1131, () => _ssrg_console_println($ssrg$atSource($ssrg$source1131, () => ((_ssrg_debug_intDebug["debug"]($ssrg$atSource($ssrg$source1131, () => identity<number>(_ssrg_int_subtract(0, 1, $ssrg$source1131)), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1182, () => _ssrg_console_println($ssrg$atSource($ssrg$source1182, () => ((_ssrg_show_intShow["show"](_ssrg_int_subtract(0, 1, $ssrg$source1182))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1216, () => _ssrg_console_println($ssrg$atSource($ssrg$source1216, () => ((__ssrg$instance$Render$0["render"](_ssrg_int_subtract(0, 2, $ssrg$source1216))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1252, () => _ssrg_console_println($ssrg$atSource($ssrg$source1252, () => ((_ssrg_show_floatShow["show"](-(2.5))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1288, () => _ssrg_console_println($ssrg$atSource($ssrg$source1288, () => ((_ssrg_debug_boolDebug["debug"](!(true))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1326, () => _ssrg_console_println($ssrg$atSource($ssrg$source1335, () => ((_ssrg_debug_intDebug["debug"](_ssrg_int_subtract(0, 3, $ssrg$source1342))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1351, () => _ssrg_console_println($ssrg$atSource($ssrg$source1360, () => ((_ssrg_debug_arrayDebug<number>(_ssrg_debug_intDebug)["debug"]([_ssrg_int_subtract(0, 4, $ssrg$source1367), _ssrg_int_subtract(0, 5, $ssrg$source1371)])) as string), false)), true), () => $ssrg$atSource($ssrg$source1380, () => _ssrg_console_println($ssrg$atSource($ssrg$source1380, () => ((_ssrg_debug_intDebug["debug"]($ssrg$atSource($ssrg$source1380, () => __ssrg$operator$3c7e3e(_ssrg_int_subtract(0, 5, $ssrg$source1380))(2), false))) as string), false)), true)))))))))
export const negative: number = _ssrg_int_subtract(0, 2, $ssrg$source201);
export const negativeZero: number = -(0.0);
export const inverted: boolean = !(true);
export const minimum: number = _ssrg_int_subtract(0, 9007199254740991, $ssrg$source293);
export const values: ReadonlyArray<number> = [_ssrg_int_subtract(0, 1, $ssrg$source342), _ssrg_int_subtract(0, 2, $ssrg$source346), _ssrg_int_subtract(0, 3, $ssrg$source350)];
export const floats: ReadonlyArray<number> = [-(1.0), -(0.0), -(6.022e23)];
export const flags: ReadonlyArray<boolean> = [!(true), !(false)];
export const tuple: readonly [number, number, boolean] = [_ssrg_int_subtract(0, 4, $ssrg$source491), -(2.5), !(false)] as const;
export const record: { readonly "inverted": boolean; readonly "negative": number } = (<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "negative": _ssrg_int_subtract(0, 5, $ssrg$source573), "inverted": !(true) } as const));
export const snapshot: Snapshot = (({ "negative": _ssrg_int_subtract(0, 6, $ssrg$source651), "negativeZero": -(0.0), "inverted": !(false) } as const) as unknown as Snapshot);

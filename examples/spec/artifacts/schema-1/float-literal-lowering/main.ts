import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { floatDebug as _ssrg_debug_floatDebug, arrayDebug as _ssrg_debug_arrayDebug, tupleDebug as _ssrg_debug_tupleDebug, recordDebug as _ssrg_debug_recordDebug, floatShow as _ssrg_show_floatShow } from "@seseragi/runtime/show"
const $ssrg$source375 = { file: "main.ssrg", line: 13, column: 5, uri: "seseragi://artifact/float-literal-lowering", start: 375, end: 397 };
const $ssrg$source384 = { file: "main.ssrg", line: 13, column: 14, uri: "seseragi://artifact/float-literal-lowering", start: 384, end: 396 };
const $ssrg$source402 = { file: "main.ssrg", line: 14, column: 5, uri: "seseragi://artifact/float-literal-lowering", start: 402, end: 422 };
const $ssrg$source411 = { file: "main.ssrg", line: 14, column: 14, uri: "seseragi://artifact/float-literal-lowering", start: 411, end: 421 };
const $ssrg$source427 = { file: "main.ssrg", line: 15, column: 5, uri: "seseragi://artifact/float-literal-lowering", start: 427, end: 449 };
const $ssrg$source436 = { file: "main.ssrg", line: 15, column: 14, uri: "seseragi://artifact/float-literal-lowering", start: 436, end: 448 };
const $ssrg$source454 = { file: "main.ssrg", line: 16, column: 5, uri: "seseragi://artifact/float-literal-lowering", start: 454, end: 472 };
const $ssrg$source463 = { file: "main.ssrg", line: 16, column: 14, uri: "seseragi://artifact/float-literal-lowering", start: 463, end: 471 };
const $ssrg$source477 = { file: "main.ssrg", line: 17, column: 5, uri: "seseragi://artifact/float-literal-lowering", start: 477, end: 512 };
const $ssrg$source486 = { file: "main.ssrg", line: 17, column: 14, uri: "seseragi://artifact/float-literal-lowering", start: 486, end: 511 };
$ssrg$assertUnicodeVersion("17.0.0")

const $ssrg$closedDisplay$0 = (() => { const create = () => _ssrg_debug_tupleDebug<readonly [number, number]>(_ssrg_debug_floatDebug, _ssrg_debug_floatDebug); let value: ReturnType<typeof create> | undefined; return (): ReturnType<typeof create> => (value ??= create()); })()
declare const __ssrg$brand$Measurement: unique symbol;
type Measurement = {
  readonly "reading": number;
  readonly [__ssrg$brand$Measurement]: true;
};
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source375, () => _ssrg_console_println($ssrg$atSource($ssrg$source384, () => ((_ssrg_debug_arrayDebug<number>(_ssrg_debug_floatDebug)["debug"](values)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source402, () => _ssrg_console_println($ssrg$atSource($ssrg$source411, () => (($ssrg$closedDisplay$0()["debug"](pair)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source427, () => _ssrg_console_println($ssrg$atSource($ssrg$source436, () => ((_ssrg_debug_recordDebug<{ readonly "value": number }>(["value"] as const, [false] as const, _ssrg_debug_floatDebug)["debug"](sample)) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source454, () => _ssrg_console_println($ssrg$atSource($ssrg$source463, () => ((_ssrg_show_floatShow["show"](1.0)) as string), false)), true), () => $ssrg$atSource($ssrg$source477, () => _ssrg_console_println($ssrg$atSource($ssrg$source486, () => ((_ssrg_debug_floatDebug["debug"]((measurement)["reading"])) as string), false)), true)))))
const values: ReadonlyArray<number> = [1.0, 2.3, -(0.0), 6.022e23];
const pair: readonly [number, number] = [1.25, 1e-9] as const;
const sample: { readonly "value": number } = (<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "value": 2.5 } as const));
const measurement: Measurement = (({ "reading": -(0.0) } as const) as unknown as Measurement);

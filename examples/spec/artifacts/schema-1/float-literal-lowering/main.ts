import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { floatDebug as _ssrg_debug_floatDebug, arrayDebug as _ssrg_debug_arrayDebug, tupleDebug as _ssrg_debug_tupleDebug, recordDebug as _ssrg_debug_recordDebug, floatShow as _ssrg_show_floatShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Measurement: unique symbol;
type Measurement = {
  readonly "reading": number;
  readonly [__ssrg$brand$Measurement]: true;
};
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_arrayDebug<number>(_ssrg_debug_floatDebug)["debug"](values)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_tupleDebug<readonly [number, number]>(_ssrg_debug_floatDebug, _ssrg_debug_floatDebug)["debug"](pair)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_recordDebug<{ readonly "value": number }>(["value"] as const, [false] as const, _ssrg_debug_floatDebug)["debug"](sample)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_show_floatShow["show"](1.0)), () => _ssrg_console_println(_ssrg_debug_floatDebug["debug"]((measurement)["reading"]))))))
const values: ReadonlyArray<number> = [1.0, 2.3, -(0.0), 6.022e23];
const pair: readonly [number, number] = [1.25, 1e-9] as const;
const sample: { readonly "value": number } = ({ "value": 2.5 } as const);
const measurement: Measurement = (({ "reading": -(0.0) } as const) as unknown as Measurement);

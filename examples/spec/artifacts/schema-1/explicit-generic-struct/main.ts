import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source206 = { file: "main.ssrg", line: 12, column: 3, uri: "seseragi://artifact/explicit-generic-struct", start: 206, end: 247 };
const $ssrg$source272 = { file: "main.ssrg", line: 15, column: 3, uri: "seseragi://artifact/explicit-generic-struct", start: 272, end: 290 };
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export interface $ssrg$kind$Marker extends $ssrg$Kind<readonly [unknown]> { readonly type: Marker<this["args"][0]> }
declare const __ssrg$brand$Marker: unique symbol;
export type Marker<A> = {
  readonly "label": string;
  readonly "value": A;
  readonly [__ssrg$brand$Marker]: true;
};
const render = (value: Marker<ReadonlyArray<string>>) => $ssrg$atSource($ssrg$source206, () => _ssrg_string_add_dictionary["add"]("Explicit generic Struct: ")((value)["label"]), false)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source272, () => _ssrg_console_println($ssrg$atSource($ssrg$source272, () => render(marker), false)), true)
export const marker: Marker<ReadonlyArray<string>> = (({ "label": "ready", "value": [] as ReadonlyArray<string> } as const) as unknown as Marker<ReadonlyArray<string>>);

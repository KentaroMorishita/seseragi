import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
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
const render = (value: Marker<ReadonlyArray<string>>) => _ssrg_string_add_dictionary["add"]("Explicit generic Struct: ")((value)["label"])
export const main = (_unit: undefined) => _ssrg_console_println(render(marker))
export const marker: Marker<ReadonlyArray<string>> = (({ "label": "ready", "value": [] as ReadonlyArray<string> } as const) as unknown as Marker<ReadonlyArray<string>>);

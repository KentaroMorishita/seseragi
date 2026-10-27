import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export interface $ssrg$kind$Box extends $ssrg$Kind<readonly [unknown]> { readonly type: Box<this["args"][0]> }
declare const __ssrg$brand$Box: unique symbol;
export type Box<A> = {
  readonly "value": A;
  readonly [__ssrg$brand$Box]: true;
};
const replace = <A,>(value: A) => (box: Box<A>) => (({ ...box, "value": value } as const) as unknown as Box<A>)
const unwrap = <A,>(box: Box<A>) => (($ssrg_match: Box<A>): A => ((value: A): A => value)($ssrg_match["value"]))(box)
const render = (value: number) => "Generic Struct: " + _ssrg_show_intShow["show"](value)
export const main = (_unit: undefined) => _ssrg_console_println(render(unwrap<number>(replace<number>(42)(inferred))))
export const inferred: Box<number> = (({ "value": 42 } as const) as unknown as Box<number>);

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source127 = { file: "project/imported-alias-return/src/domain.ssrg", line: 4, column: 40, uri: "seseragi://fixture/imported-alias-return::domain", start: 127, end: 136 };
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export interface $ssrg$kind$Again extends $ssrg$Kind<readonly [unknown]> { readonly type: Again<this["args"][0]> }
export interface $ssrg$kind$Box extends $ssrg$Kind<readonly [unknown]> { readonly type: Box<this["args"][0]> }
export interface $ssrg$kind$Wrapped extends $ssrg$Kind<readonly [unknown]> { readonly type: Wrapped<this["args"][0]> }
export type Box<A> =
  | { readonly tag: "Box"; readonly value: A };
export const Box = <A>(value: A): Box<A> => ({ tag: "Box", value } as const);
export type Wrapped<A> = Box<A>
export type Again<A> = Box<A>
export const boxed = <A,>(value: A) => $ssrg$atSource($ssrg$source127, () => Box<A>(value), false)

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source228 = { file: "project/imported-generic-adt-functor/src/domain.ssrg", line: 11, column: 21, uri: "seseragi://fixture/imported-generic-adt-functor::domain", start: 228, end: 242 };
const $ssrg$source236 = { file: "project/imported-generic-adt-functor/src/domain.ssrg", line: 11, column: 29, uri: "seseragi://fixture/imported-generic-adt-functor::domain", start: 236, end: 242 };
const $ssrg$source343 = { file: "project/imported-generic-adt-functor/src/domain.ssrg", line: 20, column: 3, uri: "seseragi://fixture/imported-generic-adt-functor::domain", start: 343, end: 354 };
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export interface $ssrg$kind$Box extends $ssrg$Kind<readonly [unknown]> { readonly type: Box<this["args"][0]> }
export type Box<A> =
  | { readonly tag: "Boxed"; readonly value: A };
export const Boxed = <A>(value: A): Box<A> => ({ tag: "Boxed", value } as const);
export const __ssrg$instance$Functor$0 = { "map": <A, B,>(f: (argument: A) => B) => (value: Box<A>) => (($ssrg_match: Box<A>): Box<B> => $ssrg_match.tag === "Boxed" ? ((item: A): Box<B> => $ssrg$atSource($ssrg$source228, () => Boxed<B>($ssrg$atSource($ssrg$source236, () => f(item), false)), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value) } as const;
export const transform = <F extends { readonly args: readonly [unknown]; readonly type: unknown }, A, B,>(f: (argument: A) => B) => (value: $ssrg$Apply<F, readonly [A]>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source343, () => ((__ssrg$evidence$0["map"](f)(value)) as $ssrg$Apply<F, readonly [B]>), false)

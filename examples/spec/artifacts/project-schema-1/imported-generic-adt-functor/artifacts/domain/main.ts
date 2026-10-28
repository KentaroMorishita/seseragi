import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export interface $ssrg$kind$Box extends $ssrg$Kind<readonly [unknown]> { readonly type: Box<this["args"][0]> }
export type Box<A> =
  | { readonly tag: "Boxed"; readonly value: A };
export const Boxed = <A>(value: A): Box<A> => ({ tag: "Boxed", value } as const);
export const __ssrg$instance$Functor$0 = { "map": <A, B,>(f: (argument: A) => B) => (value: Box<A>) => (($ssrg_match: Box<A>): Box<B> => $ssrg_match.tag === "Boxed" ? ((item: A): Box<B> => Boxed<B>(f(item)))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(value) } as const;
export const transform = <F extends { readonly args: readonly [unknown]; readonly type: unknown }, A, B,>(f: (argument: A) => B) => (value: $ssrg$Apply<F, readonly [A]>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ((__ssrg$evidence$0["map"](f)(value)) as $ssrg$Apply<F, readonly [B]>)

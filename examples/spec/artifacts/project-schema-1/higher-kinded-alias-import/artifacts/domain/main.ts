import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export interface $ssrg$kind$Box extends $ssrg$Kind<readonly [unknown]> { readonly type: Box<this["args"][0]> }
export interface $ssrg$kind$StateT extends $ssrg$Kind<readonly [unknown, { readonly args: readonly [unknown]; readonly type: unknown }, unknown]> { readonly type: StateT<this["args"][0], this["args"][1], this["args"][2]> }
export type Box<A> =
  | { readonly tag: "Boxed"; readonly value: A };
export const Boxed = <A>(value: A): Box<A> => ({ tag: "Boxed", value } as const);
export type StateT<S, M extends { readonly args: readonly [unknown]; readonly type: unknown }, A> = (argument: S) => $ssrg$Apply<M, readonly [readonly [A, S]]>
export const keepMaybe = (value: (argument: number) => { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: readonly [string, number] }) => value
export const keepEither = (value: (argument: number) => { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: readonly [number, number] }) => value

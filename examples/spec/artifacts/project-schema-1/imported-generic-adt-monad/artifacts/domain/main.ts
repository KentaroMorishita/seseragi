import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source274 = { file: "project/imported-generic-adt-monad/src/domain.ssrg", line: 14, column: 21, uri: "seseragi://fixture/imported-generic-adt-monad::domain", start: 274, end: 288 };
const $ssrg$source282 = { file: "project/imported-generic-adt-monad/src/domain.ssrg", line: 14, column: 29, uri: "seseragi://fixture/imported-generic-adt-monad::domain", start: 282, end: 288 };
const $ssrg$source499 = { file: "project/imported-generic-adt-monad/src/domain.ssrg", line: 25, column: 35, uri: "seseragi://fixture/imported-generic-adt-monad::domain", start: 499, end: 510 };
const $ssrg$source646 = { file: "project/imported-generic-adt-monad/src/domain.ssrg", line: 29, column: 39, uri: "seseragi://fixture/imported-generic-adt-monad::domain", start: 646, end: 667 };
const $ssrg$source654 = { file: "project/imported-generic-adt-monad/src/domain.ssrg", line: 29, column: 47, uri: "seseragi://fixture/imported-generic-adt-monad::domain", start: 654, end: 667 };
const $ssrg$source907 = { file: "project/imported-generic-adt-monad/src/domain.ssrg", line: 41, column: 21, uri: "seseragi://fixture/imported-generic-adt-monad::domain", start: 907, end: 913 };
const $ssrg$source1010 = { file: "project/imported-generic-adt-monad/src/domain.ssrg", line: 50, column: 3, uri: "seseragi://fixture/imported-generic-adt-monad::domain", start: 1010, end: 1025 };
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export interface $ssrg$kind$Box extends $ssrg$Kind<readonly [unknown]> { readonly type: Box<this["args"][0]> }
export type Box<A> =
  | { readonly tag: "Boxed"; readonly value: A };
export const Boxed = <A>(value: A): Box<A> => ({ tag: "Boxed", value } as const);
export type Profile =
  | { readonly tag: "Profile"; readonly value: readonly [string, number] };
export const Profile = (value: readonly [string, number]): Profile => ({ tag: "Profile", value } as const);
export const __ssrg$instance$Functor$0 = { "map": <A, B,>(f: (argument: A) => B) => (value: Box<A>) => (($ssrg_match: Box<A>): Box<B> => $ssrg_match.tag === "Boxed" ? ((item: A): Box<B> => $ssrg$atSource($ssrg$source274, () => Boxed<B>($ssrg$atSource($ssrg$source282, () => f(item), false)), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value) } as const;
export const __ssrg$instance$Applicative$1 = (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ ...__ssrg$evidence$0, "pure": <A,>(value: A) => $ssrg$atSource($ssrg$source499, () => Boxed<A>(value), false), "apply": <A, B,>(wrapped: Box<(argument: A) => B>) => (value: Box<A>) => (($ssrg_match: readonly [Box<(argument: A) => B>, Box<A>]): Box<B> => $ssrg_match[0].tag === "Boxed" && $ssrg_match[1].tag === "Boxed" ? ((_function: (argument: A) => B, item: A): Box<B> => $ssrg$atSource($ssrg$source646, () => Boxed<B>($ssrg$atSource($ssrg$source654, () => _function(item), false)), false))($ssrg_match[0].value, $ssrg_match[1].value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())([wrapped, value] as const) }) as const;
export const __ssrg$instance$Monad$2 = (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ ...__ssrg$evidence$0, "flatMap": <A, B,>(f: (argument: A) => Box<B>) => (value: Box<A>) => (($ssrg_match: Box<A>): Box<B> => $ssrg_match.tag === "Boxed" ? ((item: A): Box<B> => $ssrg$atSource($ssrg$source907, () => f(item), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value) }) as const;
export const bind = <M extends { readonly args: readonly [unknown]; readonly type: unknown }, A, B,>(f: (argument: A) => $ssrg$Apply<M, readonly [B]>) => (value: $ssrg$Apply<M, readonly [A]>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source1010, () => ((__ssrg$evidence$0["flatMap"](f)(value)) as $ssrg$Apply<M, readonly [B]>), false)

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { Just as _ssrg_maybe_Just, Right as _ssrg_either_Right } from "@seseragi/runtime/sum"
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
type Box<A> =
  | { readonly tag: "Boxed"; readonly value: A };
const Boxed = <A>(value: A): Box<A> => ({ tag: "Boxed", value } as const);
export const runOptional = (state: (argument: number) => { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: readonly [string, number] }) => state(7)
export const runEither = (state: (argument: number) => { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: readonly [string, number] }) => state(7)
export const optional: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number } = _ssrg_maybe_Just(42);
export const either: { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: number } = _ssrg_either_Right(42);
export const boxed: Box<number> = Boxed<number>(42);

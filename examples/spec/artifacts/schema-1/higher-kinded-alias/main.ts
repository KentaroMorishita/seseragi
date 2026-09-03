import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { Just as _ssrg_maybe_Just, Right as _ssrg_either_Right } from "@seseragi/runtime/sum"
$ssrg$assertUnicodeVersion("17.0.0")

type Box<A> =
  | { readonly tag: "Boxed"; readonly value: A };
const Boxed = <A>(value: A): Box<A> => ({ tag: "Boxed", value } as const);
export const runOptional = (state: (argument: number) => { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: readonly [string, number] }) => state(7)
export const runEither = (state: (argument: number) => { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: readonly [string, number] }) => state(7)
export const optional: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number } = _ssrg_maybe_Just(42);
export const either: { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: number } = _ssrg_either_Right(42);
export const boxed: Box<number> = Boxed(42);

import { Just as _ssrg_maybe_Just, Right as _ssrg_either_Right } from "@seseragi/runtime/sum"

type Box<A> =
  | { readonly tag: "Boxed"; readonly value: A };
const Boxed = <A>(value: A): Box<A> => ({ tag: "Boxed", value } as const);
export const runOptional = (state: (argument: bigint) => { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: readonly [string, bigint] }) => state(7n)
export const runEither = (state: (argument: bigint) => { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: readonly [string, bigint] }) => state(7n)
export const optional: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: bigint } = _ssrg_maybe_Just(42n);
export const either: { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: bigint } = _ssrg_either_Right(42n);
export const boxed: Box<bigint> = Boxed(42n);

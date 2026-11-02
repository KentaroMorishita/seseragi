import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { Just as _ssrg_maybe_Just, Right as _ssrg_either_Right } from "@seseragi/runtime/sum"
const $ssrg$source284 = { file: "main.ssrg", line: 12, column: 41, uri: "seseragi://artifact/higher-kinded-alias", start: 284, end: 291 };
const $ssrg$source342 = { file: "main.ssrg", line: 13, column: 51, uri: "seseragi://artifact/higher-kinded-alias", start: 342, end: 350 };
const $ssrg$source384 = { file: "main.ssrg", line: 14, column: 34, uri: "seseragi://artifact/higher-kinded-alias", start: 384, end: 392 };
const $ssrg$source477 = { file: "main.ssrg", line: 18, column: 3, uri: "seseragi://artifact/higher-kinded-alias", start: 477, end: 484 };
const $ssrg$source582 = { file: "main.ssrg", line: 22, column: 3, uri: "seseragi://artifact/higher-kinded-alias", start: 582, end: 589 };
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
type Box<A> =
  | { readonly tag: "Boxed"; readonly value: A };
const Boxed = <A>(value: A): Box<A> => ({ tag: "Boxed", value } as const);
export const runOptional = (state: (argument: number) => { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: readonly [string, number] }) => $ssrg$atSource($ssrg$source477, () => state(7), false)
export const runEither = (state: (argument: number) => { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: readonly [string, number] }) => $ssrg$atSource($ssrg$source582, () => state(7), false)
export const optional: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number } = $ssrg$atSource($ssrg$source284, () => _ssrg_maybe_Just(42), false);
export const either: { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: number } = $ssrg$atSource($ssrg$source342, () => _ssrg_either_Right(42), false);
export const boxed: Box<number> = $ssrg$atSource($ssrg$source384, () => Boxed<number>(42), false);

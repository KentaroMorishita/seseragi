import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { maybeSequence as _ssrg_maybe_sequence, eitherSequence as _ssrg_either_sequence, maybeMonoid as _ssrg_maybe_monoid } from "@seseragi/runtime/sum"
import { arrayReducible as _ssrg_array_reducible } from "@seseragi/runtime/array"
import { combine as _ssrg_collection_combine } from "@seseragi/runtime/collection"
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export const gather = <F extends { readonly args: readonly [unknown]; readonly type: unknown }, A,>(values: $ssrg$Apply<F, readonly [{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }]>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_maybe_sequence(__ssrg$evidence$0, values)
export const gatherEither = <F extends { readonly args: readonly [unknown]; readonly type: unknown }, E, A,>(values: $ssrg$Apply<F, readonly [{ readonly tag: "Left"; readonly value: E } | { readonly tag: "Right"; readonly value: A }]>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_either_sequence(__ssrg$evidence$0, values)
export const accumulated = <A,>(values: ReadonlyArray<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_collection_combine(_ssrg_array_reducible, _ssrg_maybe_monoid<A>(__ssrg$evidence$0), values)

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { maybeSequence as _ssrg_maybe_sequence, eitherSequence as _ssrg_either_sequence, maybeMonoid as _ssrg_maybe_monoid } from "@seseragi/runtime/sum"
import { arrayReducible as _ssrg_array_reducible } from "@seseragi/runtime/array"
import { combine as _ssrg_collection_combine } from "@seseragi/runtime/collection"
$ssrg$assertUnicodeVersion("17.0.0")

export const gather = <F, A,>(values: unknown) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_maybe_sequence(__ssrg$evidence$0, values)
export const gatherEither = <F, E, A,>(values: unknown) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_either_sequence(__ssrg$evidence$0, values)
export const accumulated = <A,>(values: ReadonlyArray<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_collection_combine(_ssrg_array_reducible, _ssrg_maybe_monoid<A>(__ssrg$evidence$0), values)

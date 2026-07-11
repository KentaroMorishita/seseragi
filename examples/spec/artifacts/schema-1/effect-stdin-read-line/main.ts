import { flatMap as _ssrg_effect_flatMap, succeed as _ssrg_effect_succeed } from "@seseragi/runtime/effect"
import { readLine as _ssrg_stdin_readLine } from "@seseragi/runtime/stdin"

export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_stdin_readLine(), (first: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => _ssrg_effect_flatMap(_ssrg_stdin_readLine(), (second: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => _ssrg_effect_succeed(undefined)))

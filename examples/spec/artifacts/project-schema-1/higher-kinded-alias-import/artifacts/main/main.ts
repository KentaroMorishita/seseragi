import { type Box, keepMaybe, keepEither, type $ssrg$kind$Box } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source149 = { file: "project/higher-kinded-alias-import/src/main.ssrg", line: 5, column: 3, uri: "seseragi://fixture/higher-kinded-alias-import::main", start: 149, end: 164 };
const $ssrg$source271 = { file: "project/higher-kinded-alias-import/src/main.ssrg", line: 9, column: 3, uri: "seseragi://fixture/higher-kinded-alias-import::main", start: 271, end: 287 };
$ssrg$assertUnicodeVersion("17.0.0")

export const useMaybe = (value: (argument: number) => { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: readonly [string, number] }) => $ssrg$atSource($ssrg$source149, () => keepMaybe(value), false)
export const useEither = (value: (argument: number) => { readonly tag: "Left"; readonly value: string } | { readonly tag: "Right"; readonly value: readonly [number, number] }) => $ssrg$atSource($ssrg$source271, () => keepEither(value), false)
export const useBox = (value: (argument: number) => Box<readonly [string, number]>) => value

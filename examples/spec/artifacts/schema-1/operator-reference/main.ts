import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
$ssrg$assertUnicodeVersion("17.0.0")

export const foldPair = (step: (argument: number) => (argument: number) => number) => (initial: number) => (value: number) => step(initial)(value)
export const addPair = (initial: number) => (value: number) => foldPair((((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number))(initial)(value)

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { add as _ssrg_int_add, subtract as _ssrg_int_subtract, multiply as _ssrg_int_multiply, divide as _ssrg_int_divide, remainder as _ssrg_int_remainder, power as _ssrg_int_power } from "@seseragi/runtime/int"
$ssrg$assertUnicodeVersion("17.0.0")

export const add = (left: number) => (right: number) => _ssrg_int_add(left, right)
export const subtract = (left: number) => (right: number) => _ssrg_int_subtract(left, right)
export const multiply = (left: number) => (right: number) => _ssrg_int_multiply(left, right)
export const divide = (left: number) => (right: number) => _ssrg_int_divide(left, right)
export const remainder = (left: number) => (right: number) => _ssrg_int_remainder(left, right)
export const power = (base: number) => (exponent: number) => _ssrg_int_power(base, exponent)

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add, subtract as _ssrg_int_subtract, multiply as _ssrg_int_multiply, divide as _ssrg_int_divide, remainder as _ssrg_int_remainder, power as _ssrg_int_power } from "@seseragi/runtime/int"
const $ssrg$source44 = { file: "main.ssrg", line: 1, column: 45, uri: "seseragi://artifact/int-arithmetic", start: 44, end: 56 };
const $ssrg$source106 = { file: "main.ssrg", line: 2, column: 50, uri: "seseragi://artifact/int-arithmetic", start: 106, end: 118 };
const $ssrg$source168 = { file: "main.ssrg", line: 3, column: 50, uri: "seseragi://artifact/int-arithmetic", start: 168, end: 180 };
const $ssrg$source228 = { file: "main.ssrg", line: 4, column: 48, uri: "seseragi://artifact/int-arithmetic", start: 228, end: 240 };
const $ssrg$source291 = { file: "main.ssrg", line: 5, column: 51, uri: "seseragi://artifact/int-arithmetic", start: 291, end: 303 };
const $ssrg$source353 = { file: "main.ssrg", line: 6, column: 50, uri: "seseragi://artifact/int-arithmetic", start: 353, end: 369 };
$ssrg$assertUnicodeVersion("17.0.0")

export const add = (left: number) => (right: number) => _ssrg_int_add(left, right, $ssrg$source44)
export const subtract = (left: number) => (right: number) => _ssrg_int_subtract(left, right, $ssrg$source106)
export const multiply = (left: number) => (right: number) => _ssrg_int_multiply(left, right, $ssrg$source168)
export const divide = (left: number) => (right: number) => _ssrg_int_divide(left, right, $ssrg$source228)
export const remainder = (left: number) => (right: number) => _ssrg_int_remainder(left, right, $ssrg$source291)
export const power = (base: number) => (exponent: number) => _ssrg_int_power(base, exponent, $ssrg$source353)

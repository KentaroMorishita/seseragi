import { add as _ssrg_int64_add, subtract as _ssrg_int64_subtract, multiply as _ssrg_int64_multiply, divide as _ssrg_int64_divide, remainder as _ssrg_int64_remainder, power as _ssrg_int64_power } from "@seseragi/runtime/int64"

export const add = (left: bigint) => (right: bigint) => _ssrg_int64_add(left, right)
export const subtract = (left: bigint) => (right: bigint) => _ssrg_int64_subtract(left, right)
export const multiply = (left: bigint) => (right: bigint) => _ssrg_int64_multiply(left, right)
export const divide = (left: bigint) => (right: bigint) => _ssrg_int64_divide(left, right)
export const remainder = (left: bigint) => (right: bigint) => _ssrg_int64_remainder(left, right)
export const power = (base: bigint) => (exponent: bigint) => _ssrg_int64_power(base, exponent)

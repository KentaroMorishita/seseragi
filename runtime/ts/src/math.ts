// Pure mathematical operations; the portable contract is in spec section 10.
export const pi = Math.PI
export const e = Math.E
export const tau = 6.283185307179586

export const sin = (value: number): number => Math.sin(value)
export const cos = (value: number): number => Math.cos(value)
export const tan = (value: number): number => Math.tan(value)
export const asin = (value: number): number => Math.asin(value)
export const acos = (value: number): number => Math.acos(value)
export const atan = (value: number): number => Math.atan(value)
export const exp = (value: number): number => Math.exp(value)
export const log = (value: number): number => Math.log(value)
export const log2 = (value: number): number => Math.log2(value)
export const log10 = (value: number): number => Math.log10(value)
export const sqrt = (value: number): number => Math.sqrt(value)
export const cbrt = (value: number): number => Math.cbrt(value)
export const sinh = (value: number): number => Math.sinh(value)
export const cosh = (value: number): number => Math.cosh(value)
export const tanh = (value: number): number => Math.tanh(value)

export const atan2 = (y: number, x: number): number => Math.atan2(y, x)
export const hypot = (x: number, y: number): number => Math.hypot(x, y)

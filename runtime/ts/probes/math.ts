import * as math from "../src/math"

function same(actual: number, expected: number): void {
  if (!Object.is(actual, expected))
    throw new Error(`math boundary: ${actual} != ${expected}`)
}
function near(actual: number, expected: number): void {
  if (
    !Number.isFinite(actual) ||
    Math.abs(actual - expected) > 1e-14 * Math.max(1, Math.abs(expected))
  )
    throw new Error(`math approximation: ${actual} != ${expected}`)
}

// biome-ignore lint/suspicious/noApproximativeNumericConstant: independent reference value
same(math.pi, 3.141592653589793)
// biome-ignore lint/suspicious/noApproximativeNumericConstant: independent reference value
same(math.e, 2.718281828459045)
same(math.tau, 6.283185307179586)
for (const fn of [
  math.sin,
  math.tan,
  math.asin,
  math.atan,
  math.sqrt,
  math.cbrt,
  math.sinh,
  math.tanh,
]) {
  same(fn(-0), -0)
  same(fn(0), 0)
}
for (const fn of [
  math.sin,
  math.cos,
  math.tan,
  math.asin,
  math.acos,
  math.atan,
  math.exp,
  math.log,
  math.log2,
  math.log10,
  math.sqrt,
  math.cbrt,
  math.sinh,
  math.cosh,
  math.tanh,
])
  same(fn(NaN), NaN)
for (const fn of [math.sin, math.cos, math.tan]) {
  same(fn(Infinity), NaN)
  same(fn(-Infinity), NaN)
}
for (const fn of [math.asin, math.acos]) {
  same(fn(1.0000000000000002), NaN)
  same(fn(-1.0000000000000002), NaN)
}
same(math.asin(1), math.pi / 2)
same(math.acos(1), 0)
same(math.acos(-1), math.pi)
same(math.atan(Infinity), math.pi / 2)
same(math.atan(-Infinity), -math.pi / 2)
same(math.exp(Infinity), Infinity)
same(math.exp(-Infinity), 0)
same(math.exp(1000), Infinity)
same(math.exp(-1000), 0)
for (const fn of [math.log, math.log2, math.log10]) {
  same(fn(0), -Infinity)
  same(fn(-0), -Infinity)
  same(fn(-1), NaN)
  same(fn(1), 0)
  same(fn(Infinity), Infinity)
}
same(math.sqrt(-1), NaN)
same(math.sqrt(Infinity), Infinity)
same(math.cbrt(-Infinity), -Infinity)
same(math.sinh(-Infinity), -Infinity)
same(math.cosh(-Infinity), Infinity)
same(math.tanh(-Infinity), -1)
same(math.tanh(Infinity), 1)
same(math.hypot(NaN, Infinity), Infinity)
same(math.hypot(-Infinity, NaN), Infinity)
same(math.hypot(NaN, 1), NaN)
same(math.hypot(-0, 0), 0)
same(math.atan2(NaN, 1), NaN)
same(math.atan2(1, NaN), NaN)
for (const sign of [1, -1]) {
  same(math.atan2(sign * 0, 0), sign * 0)
  same(math.atan2(sign * 0, -0), sign * math.pi)
  same(math.atan2(sign * 0, -1), sign * math.pi)
  same(math.atan2(sign, 0), (sign * math.pi) / 2)
  same(math.atan2(sign, Infinity), sign * 0)
  same(math.atan2(sign, -Infinity), sign * math.pi)
  same(math.atan2(sign * Infinity, Infinity), (sign * math.pi) / 4)
  same(math.atan2(sign * Infinity, -Infinity), (sign * 3 * math.pi) / 4)
}
near(math.sin(0.5), 0.479425538604203)
near(math.cos(0.5), 0.8775825618903728)
near(math.tan(0.5), 0.5463024898437905)
near(math.asin(0.5), 0.5235987755982989)
near(math.acos(0.5), 1.0471975511965979)
near(math.atan(1), 0.7853981633974483)
near(math.atan2(1, 1), 0.7853981633974483)
near(math.exp(1), math.e)
// biome-ignore lint/suspicious/noApproximativeNumericConstant: independent reference value
near(math.log(2), 0.6931471805599453)
near(math.log2(8), 3)
near(math.log10(100), 2)
// biome-ignore lint/suspicious/noApproximativeNumericConstant: independent reference value
near(math.sqrt(2), 1.4142135623730951)
near(math.cbrt(-8), -2)
near(math.hypot(3, 4), 5)
near(math.sinh(1), 1.1752011936438014)
near(math.cosh(1), 1.5430806348152437)
near(math.tanh(1), 0.7615941559557649)
near(math.hypot(3e200, 4e200) / 1e200, 5)
near(math.hypot(3e-200, 4e-200) / 1e-200, 5)
if (math.sin(Number.MIN_VALUE) === 0 || math.hypot(Number.MIN_VALUE, 0) === 0)
  throw new Error("math flushed a representable subnormal")

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { pi as _ssrg_math_pi, e as _ssrg_math_e, tau as _ssrg_math_tau, sin as _ssrg_math_sin, cos as _ssrg_math_cos, tan as _ssrg_math_tan, asin as _ssrg_math_asin, acos as _ssrg_math_acos, atan as _ssrg_math_atan, exp as _ssrg_math_exp, log as _ssrg_math_log, log2 as _ssrg_math_log2, log10 as _ssrg_math_log10, sqrt as _ssrg_math_sqrt, cbrt as _ssrg_math_cbrt, sinh as _ssrg_math_sinh, cosh as _ssrg_math_cosh, tanh as _ssrg_math_tanh, atan2 as _ssrg_math_atan2, hypot as _ssrg_math_hypot } from "@seseragi/runtime/math"
$ssrg$assertUnicodeVersion("17.0.0")

export const constants: readonly [number, number, number, number] = [_ssrg_math_pi, _ssrg_math_e, _ssrg_math_tau, _ssrg_math_pi] as const;
export const functions: ReadonlyArray<(argument: number) => number> = [_ssrg_math_sin, _ssrg_math_cos, _ssrg_math_tan, _ssrg_math_asin, _ssrg_math_acos, _ssrg_math_atan, _ssrg_math_exp, _ssrg_math_log, _ssrg_math_log2, _ssrg_math_log10, _ssrg_math_sqrt, _ssrg_math_cbrt, _ssrg_math_sinh, _ssrg_math_cosh, _ssrg_math_tanh];
export const partial: (argument: number) => number = (__ssrg$numeric$partial$0: number) => _ssrg_math_atan2(1.0, __ssrg$numeric$partial$0);
export const length: number = _ssrg_math_hypot(3.0, 4.0);
export const result: number = partial(1.0);

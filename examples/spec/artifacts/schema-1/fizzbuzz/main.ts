import { remainder as _ssrg_int_remainder } from "@seseragi/runtime/int"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { collectMap as _ssrg_range_comprehend, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"

const fizzBuzz = (number: number) => (($ssrg_match: readonly [number, number]): string => $ssrg_match[0] === 0 && $ssrg_match[1] === 0 ? "FizzBuzz" : $ssrg_match[0] === 0 ? "Fizz" : $ssrg_match[1] === 0 ? "Buzz" : _ssrg_show_intShow["show"](number))([_ssrg_int_remainder(number, 3), _ssrg_int_remainder(number, 5)] as const)
export const fizzBuzzValues = (unit: undefined) => _ssrg_range_comprehend(_ssrg_range_inclusive(1, 30), (number) => true, (number) => fizzBuzz(number))

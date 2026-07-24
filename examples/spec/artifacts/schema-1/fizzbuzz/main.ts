import { remainder as _ssrg_int64_remainder } from "@seseragi/runtime/int64"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { collectMap as _ssrg_range_comprehend, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"

const fizzBuzz = (number: bigint) => (($ssrg_match: readonly [bigint, bigint]): string => $ssrg_match[0] === 0n && $ssrg_match[1] === 0n ? "FizzBuzz" : $ssrg_match[0] === 0n ? "Fizz" : $ssrg_match[1] === 0n ? "Buzz" : _ssrg_show_intShow["show"](number))([_ssrg_int64_remainder(number, 3n), _ssrg_int64_remainder(number, 5n)] as const)
export const fizzBuzzValues = (unit: undefined) => _ssrg_range_comprehend(_ssrg_range_inclusive(1n, 30n), (number) => true, (number) => fizzBuzz(number))

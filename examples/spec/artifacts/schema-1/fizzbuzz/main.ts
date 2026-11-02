import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { remainder as _ssrg_int_remainder } from "@seseragi/runtime/int"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { collectMap as _ssrg_range_comprehend, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
const $ssrg$source45 = { file: "main.ssrg", line: 2, column: 10, uri: "seseragi://artifact/fizzbuzz", start: 45, end: 55 };
const $ssrg$source57 = { file: "main.ssrg", line: 2, column: 22, uri: "seseragi://artifact/fizzbuzz", start: 57, end: 67 };
const $ssrg$source219 = { file: "main.ssrg", line: 10, column: 4, uri: "seseragi://artifact/fizzbuzz", start: 219, end: 234 };
const $ssrg$source247 = { file: "main.ssrg", line: 10, column: 32, uri: "seseragi://artifact/fizzbuzz", start: 247, end: 253 };
$ssrg$assertUnicodeVersion("17.0.0")

const fizzBuzz = (number: number) => (($ssrg_match: readonly [number, number]): string => $ssrg_match[0] === 0 && $ssrg_match[1] === 0 ? "FizzBuzz" : $ssrg_match[0] === 0 ? "Fizz" : $ssrg_match[1] === 0 ? "Buzz" : _ssrg_show_intShow["show"](number))([_ssrg_int_remainder(number, 3, $ssrg$source45), _ssrg_int_remainder(number, 5, $ssrg$source57)] as const)
export const fizzBuzzValues = (unit: undefined) => _ssrg_range_comprehend($ssrg$atSource($ssrg$source247, () => _ssrg_range_inclusive(1, 30), false), (number) => true, (number) => $ssrg$atSource($ssrg$source219, () => fizzBuzz(number), false))

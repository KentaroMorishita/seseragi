import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { flatMap as _ssrg_effect_flatMap, type Effect as Effect } from "@seseragi/runtime/effect"
import { arrayIterable as _ssrg_array_iterable } from "@seseragi/runtime/array"
import { forEach as _ssrg_collection_for_each } from "@seseragi/runtime/collection"
import { println as _ssrg_console_println, type Console as Console, type ConsoleError as ConsoleError } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { listIterable as _ssrg_list_iterable, fromArray as _ssrg_list_from_array } from "@seseragi/runtime/list"
import { rangeIterable as _ssrg_range_iterable, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
const $ssrg$source32 = { file: "main.ssrg", line: 3, column: 5, uri: "seseragi://artifact/effectful-for", start: 32, end: 92 };
const $ssrg$source60 = { file: "main.ssrg", line: 4, column: 7, uri: "seseragi://artifact/effectful-for", start: 60, end: 86 };
const $ssrg$source97 = { file: "main.ssrg", line: 6, column: 5, uri: "seseragi://artifact/effectful-for", start: 97, end: 157 };
const $ssrg$source126 = { file: "main.ssrg", line: 7, column: 7, uri: "seseragi://artifact/effectful-for", start: 126, end: 151 };
const $ssrg$source162 = { file: "main.ssrg", line: 9, column: 5, uri: "seseragi://artifact/effectful-for", start: 162, end: 221 };
const $ssrg$source175 = { file: "main.ssrg", line: 9, column: 18, uri: "seseragi://artifact/effectful-for", start: 175, end: 180 };
const $ssrg$source189 = { file: "main.ssrg", line: 10, column: 7, uri: "seseragi://artifact/effectful-for", start: 189, end: 215 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source32, () => _ssrg_collection_for_each(_ssrg_array_iterable, ($ssrg_for_value: number) => (($ssrg_match: number): Effect<{ readonly "console": Console }, ConsoleError, undefined> => ((value: number): Effect<{ readonly "console": Console }, ConsoleError, undefined> => $ssrg$atSource($ssrg$source60, () => _ssrg_console_println("array:" + _ssrg_show_intShow["show"](value)), true))($ssrg_match))($ssrg_for_value), [1, 2]), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source97, () => _ssrg_collection_for_each(_ssrg_list_iterable, ($ssrg_for_value: number) => (($ssrg_match: number): Effect<{ readonly "console": Console }, ConsoleError, undefined> => ((value: number): Effect<{ readonly "console": Console }, ConsoleError, undefined> => $ssrg$atSource($ssrg$source126, () => _ssrg_console_println("list:" + _ssrg_show_intShow["show"](value)), true))($ssrg_match))($ssrg_for_value), _ssrg_list_from_array([3, 4])), true), () => $ssrg$atSource($ssrg$source162, () => _ssrg_collection_for_each(_ssrg_range_iterable, ($ssrg_for_value: number) => (($ssrg_match: number): Effect<{ readonly "console": Console }, ConsoleError, undefined> => ((value: number): Effect<{ readonly "console": Console }, ConsoleError, undefined> => $ssrg$atSource($ssrg$source189, () => _ssrg_console_println("range:" + _ssrg_show_intShow["show"](value)), true))($ssrg_match))($ssrg_for_value), $ssrg$atSource($ssrg$source175, () => _ssrg_range_inclusive(5, 6), false)), true)))

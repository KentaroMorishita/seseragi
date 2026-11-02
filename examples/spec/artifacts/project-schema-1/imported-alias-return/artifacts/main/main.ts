import { type Box as Foreign, type $ssrg$kind$Box as $ssrg$kind$Foreign } from "./domain.js"
import { boxed } from "./facade.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow, boolShow as _ssrg_show_boolShow } from "@seseragi/runtime/show"
const $ssrg$source187 = { file: "project/imported-alias-return/src/main.ssrg", line: 5, column: 34, uri: "seseragi://fixture/imported-alias-return::main", start: 187, end: 197 };
const $ssrg$source280 = { file: "project/imported-alias-return/src/main.ssrg", line: 7, column: 14, uri: "seseragi://fixture/imported-alias-return::main", start: 280, end: 299 };
const $ssrg$source289 = { file: "project/imported-alias-return/src/main.ssrg", line: 7, column: 23, uri: "seseragi://fixture/imported-alias-return::main", start: 289, end: 299 };
const $ssrg$source328 = { file: "project/imported-alias-return/src/main.ssrg", line: 9, column: 3, uri: "seseragi://fixture/imported-alias-return::main", start: 328, end: 353 };
const $ssrg$source337 = { file: "project/imported-alias-return/src/main.ssrg", line: 9, column: 12, uri: "seseragi://fixture/imported-alias-return::main", start: 337, end: 352 };
const $ssrg$source343 = { file: "project/imported-alias-return/src/main.ssrg", line: 9, column: 18, uri: "seseragi://fixture/imported-alias-return::main", start: 343, end: 351 };
const $ssrg$source368 = { file: "project/imported-alias-return/src/main.ssrg", line: 10, column: 15, uri: "seseragi://fixture/imported-alias-return::main", start: 368, end: 382 };
const $ssrg$source374 = { file: "project/imported-alias-return/src/main.ssrg", line: 10, column: 21, uri: "seseragi://fixture/imported-alias-return::main", start: 374, end: 381 };
const $ssrg$source385 = { file: "project/imported-alias-return/src/main.ssrg", line: 11, column: 3, uri: "seseragi://fixture/imported-alias-return::main", start: 385, end: 409 };
const $ssrg$source394 = { file: "project/imported-alias-return/src/main.ssrg", line: 11, column: 12, uri: "seseragi://fixture/imported-alias-return::main", start: 394, end: 408 };
const $ssrg$source436 = { file: "project/imported-alias-return/src/main.ssrg", line: 12, column: 27, uri: "seseragi://fixture/imported-alias-return::main", start: 436, end: 454 };
const $ssrg$source442 = { file: "project/imported-alias-return/src/main.ssrg", line: 12, column: 33, uri: "seseragi://fixture/imported-alias-return::main", start: 442, end: 453 };
const $ssrg$source457 = { file: "project/imported-alias-return/src/main.ssrg", line: 13, column: 3, uri: "seseragi://fixture/imported-alias-return::main", start: 457, end: 477 };
const $ssrg$source466 = { file: "project/imported-alias-return/src/main.ssrg", line: 13, column: 12, uri: "seseragi://fixture/imported-alias-return::main", start: 466, end: 476 };
const $ssrg$source480 = { file: "project/imported-alias-return/src/main.ssrg", line: 14, column: 3, uri: "seseragi://fixture/imported-alias-return::main", start: 480, end: 502 };
const $ssrg$source489 = { file: "project/imported-alias-return/src/main.ssrg", line: 14, column: 12, uri: "seseragi://fixture/imported-alias-return::main", start: 489, end: 501 };
$ssrg$assertUnicodeVersion("17.0.0")

type Box<A> =
  | { readonly tag: "Local"; readonly value: A };
const Local = <A>(value: A): Box<A> => ({ tag: "Local", value } as const);
const read = <A,>(value: Foreign<A>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => (($ssrg_match: Foreign<A>): string => $ssrg_match.tag === "Box" ? ((inner: A): string => $ssrg$atSource($ssrg$source187, () => ((__ssrg$evidence$0["show"](inner)) as string), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value)
const hold = <A,>(value: Foreign<A>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => (prefix: string) => $ssrg$atSource($ssrg$source280, () => _ssrg_string_add_dictionary["add"](prefix)($ssrg$atSource($ssrg$source289, () => ((read<A>(value)(__ssrg$evidence$0)) as string), false)), false)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source328, () => _ssrg_console_println($ssrg$atSource($ssrg$source337, () => ((read<number>($ssrg$atSource($ssrg$source343, () => boxed<number>(42), false))(_ssrg_show_intShow)) as string), false)), true), () => (() => { const saved: (argument: string) => string = $ssrg$atSource($ssrg$source368, () => ((hold<number>($ssrg$atSource($ssrg$source374, () => boxed<number>(7), false))(_ssrg_show_intShow)) as (argument: string) => string), false); return _ssrg_effect_flatMap($ssrg$atSource($ssrg$source385, () => _ssrg_console_println($ssrg$atSource($ssrg$source394, () => saved("saved="), false)), true), () => ((forward) => { return _ssrg_effect_flatMap($ssrg$atSource($ssrg$source457, () => _ssrg_console_println($ssrg$atSource($ssrg$source466, () => ((forward<number>(42)(_ssrg_show_intShow)) as string), false)), true), () => $ssrg$atSource($ssrg$source480, () => _ssrg_console_println($ssrg$atSource($ssrg$source489, () => ((forward<boolean>(true)(_ssrg_show_boolShow)) as string), false)), true)); })(<Let426T0,>(value: Let426T0) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source436, () => ((read<Let426T0>($ssrg$atSource($ssrg$source442, () => boxed<Let426T0>(value), false))(__ssrg$evidence$0)) as string), false))); })())

import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add, subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
const $ssrg$source145 = { file: "main.ssrg", line: 6, column: 19, uri: "seseragi://artifact/match-shapes", start: 145, end: 156 };
const $ssrg$source365 = { file: "main.ssrg", line: 17, column: 31, uri: "seseragi://artifact/match-shapes", start: 365, end: 375 };
const $ssrg$source545 = { file: "main.ssrg", line: 27, column: 3, uri: "seseragi://artifact/match-shapes", start: 545, end: 568 };
const $ssrg$source552 = { file: "main.ssrg", line: 27, column: 10, uri: "seseragi://artifact/match-shapes", start: 552, end: 567 };
const $ssrg$source560 = { file: "main.ssrg", line: 27, column: 18, uri: "seseragi://artifact/match-shapes", start: 560, end: 566 };
const $ssrg$source577 = { file: "main.ssrg", line: 27, column: 35, uri: "seseragi://artifact/match-shapes", start: 577, end: 588 };
const $ssrg$source585 = { file: "main.ssrg", line: 27, column: 43, uri: "seseragi://artifact/match-shapes", start: 585, end: 587 };
const $ssrg$source600 = { file: "main.ssrg", line: 28, column: 3, uri: "seseragi://artifact/match-shapes", start: 600, end: 614 };
const $ssrg$source623 = { file: "main.ssrg", line: 28, column: 26, uri: "seseragi://artifact/match-shapes", start: 623, end: 638 };
const $ssrg$source649 = { file: "main.ssrg", line: 29, column: 3, uri: "seseragi://artifact/match-shapes", start: 649, end: 664 };
$ssrg$assertUnicodeVersion("17.0.0")

type Tree =
  | { readonly tag: "Leaf"; readonly value: number }
  | { readonly tag: "Branch"; readonly value: Tree };
const Leaf = (value: number): Tree => ({ tag: "Leaf", value } as const);
const Branch = (value: Tree): Tree => ({ tag: "Branch", value } as const);
type Flag =
  | { readonly tag: "On" }
  | { readonly tag: "Off" };
const On: Flag = { tag: "On" } as const;
const Off: Flag = { tag: "Off" } as const;
const value = (tree: Tree) => { const $ssrg$tail = Symbol(); let $ssrg$loopArguments: readonly [Tree] = [tree]; while (true) { const tree: Tree = $ssrg$loopArguments[0]; const $ssrg$result = (($ssrg_match: Tree): number => $ssrg_match.tag === "Leaf" ? ((number: number): number => number)($ssrg_match.value) : $ssrg_match.tag === "Branch" ? ((child: Tree): number => $ssrg$atSource($ssrg$source145, () => ({ [$ssrg$tail]: [child] } as never), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(tree); const $ssrg$tailResult: unknown = $ssrg$result; if (typeof $ssrg$tailResult === "object" && $ssrg$tailResult !== null && $ssrg$tail in $ssrg$tailResult) { const $ssrg$arguments = ($ssrg$tailResult as { readonly [$ssrg$tail]: readonly [Tree] })[$ssrg$tail]; $ssrg$loopArguments = $ssrg$arguments; continue; } return $ssrg$result; } }
const signed = (number: number) => (($ssrg_match: number): number => $ssrg_match === -1 ? 10 : 20)(number)
const pair = (values: ReadonlyArray<readonly [Flag, number]>) => (($ssrg_match: ReadonlyArray<readonly [Flag, number]>): number => $ssrg_match.length === 0 ? 0 : $ssrg_match.length >= 1 && $ssrg_match[0][0].tag === "On" ? ((number: number, tail: ReadonlyArray<readonly [Flag, number]>): number => number)($ssrg_match[0][1], $ssrg_match.slice(1)) : ((number: number, tail: ReadonlyArray<readonly [Flag, number]>): number => _ssrg_int_add(number, 1, $ssrg$source365))($ssrg_match[0][1], $ssrg_match.slice(1)))(values)
const nested = (values: ReadonlyArray<ReadonlyArray<number>>) => (($ssrg_match: ReadonlyArray<ReadonlyArray<number>>): number => $ssrg_match.length === 0 ? 0 : $ssrg_match.length >= 1 && $ssrg_match[0].length === 0 ? ((tail: ReadonlyArray<ReadonlyArray<number>>): number => 1)($ssrg_match.slice(1)) : ((head: number, rest: ReadonlyArray<number>, tail: ReadonlyArray<ReadonlyArray<number>>): number => head)($ssrg_match[0][0], $ssrg_match[0].slice(1), $ssrg_match.slice(1)))(values)
export const matchShapesWork = (_unit: undefined) => ((($ssrg$atSource($ssrg$source545, () => _ssrg_int_eq_dictionary["eq"]($ssrg$atSource($ssrg$source545, () => value($ssrg$atSource($ssrg$source552, () => Branch($ssrg$atSource($ssrg$source560, () => Leaf(3), false)), false)), false))(3), false) ? $ssrg$atSource($ssrg$source577, () => _ssrg_int_eq_dictionary["eq"]($ssrg$atSource($ssrg$source577, () => signed(_ssrg_int_subtract(0, 1, $ssrg$source585)), false))(10), false) : false) ? $ssrg$atSource($ssrg$source600, () => _ssrg_int_eq_dictionary["eq"]($ssrg$atSource($ssrg$source600, () => pair([[On, 5] as const]), false))(5), false) : false) ? $ssrg$atSource($ssrg$source623, () => _ssrg_int_eq_dictionary["eq"]($ssrg$atSource($ssrg$source623, () => pair([[Off, 5] as const]), false))(6), false) : false) ? $ssrg$atSource($ssrg$source649, () => _ssrg_int_eq_dictionary["eq"]($ssrg$atSource($ssrg$source649, () => nested([[7, 8]]), false))(7), false) : false

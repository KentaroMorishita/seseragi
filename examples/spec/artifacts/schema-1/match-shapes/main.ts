import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { add as _ssrg_int_add, subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
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
const value = (tree: Tree) => { const $ssrg$tail = Symbol(); let $ssrg$loopArguments: readonly [Tree] = [tree]; while (true) { const tree: Tree = $ssrg$loopArguments[0]; const $ssrg$result = (($ssrg_match: Tree): number => $ssrg_match.tag === "Leaf" ? ((number: number): number => number)($ssrg_match.value) : $ssrg_match.tag === "Branch" ? ((child: Tree): number => ({ [$ssrg$tail]: [child] } as never))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(tree); const $ssrg$tailResult: unknown = $ssrg$result; if (typeof $ssrg$tailResult === "object" && $ssrg$tailResult !== null && $ssrg$tail in $ssrg$tailResult) { const $ssrg$arguments = ($ssrg$tailResult as { readonly [$ssrg$tail]: readonly [Tree] })[$ssrg$tail]; $ssrg$loopArguments = $ssrg$arguments; continue; } return $ssrg$result; } }
const signed = (number: number) => (($ssrg_match: number): number => $ssrg_match === -1 ? 10 : 20)(number)
const pair = (values: ReadonlyArray<readonly [Flag, number]>) => (($ssrg_match: ReadonlyArray<readonly [Flag, number]>): number => $ssrg_match.length === 0 ? 0 : $ssrg_match.length >= 1 && $ssrg_match[0][0].tag === "On" ? ((number: number, tail: ReadonlyArray<readonly [Flag, number]>): number => number)($ssrg_match[0][1], $ssrg_match.slice(1)) : ((number: number, tail: ReadonlyArray<readonly [Flag, number]>): number => _ssrg_int_add(number, 1))($ssrg_match[0][1], $ssrg_match.slice(1)))(values)
const nested = (values: ReadonlyArray<ReadonlyArray<number>>) => (($ssrg_match: ReadonlyArray<ReadonlyArray<number>>): number => $ssrg_match.length === 0 ? 0 : $ssrg_match.length >= 1 && $ssrg_match[0].length === 0 ? ((tail: ReadonlyArray<ReadonlyArray<number>>): number => 1)($ssrg_match.slice(1)) : ((head: number, rest: ReadonlyArray<number>, tail: ReadonlyArray<ReadonlyArray<number>>): number => head)($ssrg_match[0][0], $ssrg_match[0].slice(1), $ssrg_match.slice(1)))(values)
export const matchShapesWork = (_unit: undefined) => (((_ssrg_int_eq_dictionary["eq"](value(Branch(Leaf(3))))(3) ? _ssrg_int_eq_dictionary["eq"](signed(_ssrg_int_subtract(0, 1)))(10) : false) ? _ssrg_int_eq_dictionary["eq"](pair([[On, 5] as const]))(5) : false) ? _ssrg_int_eq_dictionary["eq"](pair([[Off, 5] as const]))(6) : false) ? _ssrg_int_eq_dictionary["eq"](nested([[7, 8]]))(7) : false

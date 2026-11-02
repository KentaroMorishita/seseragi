import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { subtract as _ssrg_int_subtract, add as _ssrg_int_add, intZero as _ssrg_int_zero, intAdd as _ssrg_int_add_dictionary, intOne as _ssrg_int_one, intMul as _ssrg_int_mul_dictionary } from "@seseragi/runtime/int"
import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { combine as _ssrg_collection_combine } from "@seseragi/runtime/collection"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { arrayReducible as _ssrg_array_reducible } from "@seseragi/runtime/array"
import { sumMonoid as _ssrg_sum_monoid, Sum as _ssrg_sum_Sum, productMonoid as _ssrg_product_monoid, Product as _ssrg_product_Product, sumSemigroup as _ssrg_sum_semigroup, type Sum as Sum, type Product as Product } from "@seseragi/runtime/sum"
import { listReducible as _ssrg_list_reducible, fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"
const $ssrg$source102 = { file: "main.ssrg", line: 3, column: 48, uri: "seseragi://artifact/monoid-wrappers", start: 102, end: 105 };
const $ssrg$source213 = { file: "main.ssrg", line: 7, column: 12, uri: "seseragi://artifact/monoid-wrappers", start: 213, end: 237 };
const $ssrg$source483 = { file: "main.ssrg", line: 16, column: 12, uri: "seseragi://artifact/monoid-wrappers", start: 483, end: 507 };
const $ssrg$source586 = { file: "main.ssrg", line: 19, column: 73, uri: "seseragi://artifact/monoid-wrappers", start: 586, end: 600 };
const $ssrg$source778 = { file: "main.ssrg", line: 24, column: 3, uri: "seseragi://artifact/monoid-wrappers", start: 778, end: 837 };
const $ssrg$source788 = { file: "main.ssrg", line: 24, column: 13, uri: "seseragi://artifact/monoid-wrappers", start: 788, end: 837 };
const $ssrg$source795 = { file: "main.ssrg", line: 24, column: 20, uri: "seseragi://artifact/monoid-wrappers", start: 795, end: 837 };
const $ssrg$source807 = { file: "main.ssrg", line: 24, column: 32, uri: "seseragi://artifact/monoid-wrappers", start: 807, end: 837 };
const $ssrg$source817 = { file: "main.ssrg", line: 24, column: 42, uri: "seseragi://artifact/monoid-wrappers", start: 817, end: 822 };
const $ssrg$source824 = { file: "main.ssrg", line: 24, column: 49, uri: "seseragi://artifact/monoid-wrappers", start: 824, end: 829 };
const $ssrg$source831 = { file: "main.ssrg", line: 24, column: 56, uri: "seseragi://artifact/monoid-wrappers", start: 831, end: 836 };
const $ssrg$source840 = { file: "main.ssrg", line: 25, column: 3, uri: "seseragi://artifact/monoid-wrappers", start: 840, end: 916 };
const $ssrg$source850 = { file: "main.ssrg", line: 25, column: 13, uri: "seseragi://artifact/monoid-wrappers", start: 850, end: 916 };
const $ssrg$source857 = { file: "main.ssrg", line: 25, column: 20, uri: "seseragi://artifact/monoid-wrappers", start: 857, end: 916 };
const $ssrg$source873 = { file: "main.ssrg", line: 25, column: 36, uri: "seseragi://artifact/monoid-wrappers", start: 873, end: 916 };
const $ssrg$source884 = { file: "main.ssrg", line: 25, column: 47, uri: "seseragi://artifact/monoid-wrappers", start: 884, end: 893 };
const $ssrg$source895 = { file: "main.ssrg", line: 25, column: 58, uri: "seseragi://artifact/monoid-wrappers", start: 895, end: 904 };
const $ssrg$source906 = { file: "main.ssrg", line: 25, column: 69, uri: "seseragi://artifact/monoid-wrappers", start: 906, end: 915 };
const $ssrg$source1001 = { file: "main.ssrg", line: 28, column: 3, uri: "seseragi://artifact/monoid-wrappers", start: 1001, end: 1047 };
const $ssrg$source1011 = { file: "main.ssrg", line: 28, column: 13, uri: "seseragi://artifact/monoid-wrappers", start: 1011, end: 1047 };
const $ssrg$source1018 = { file: "main.ssrg", line: 28, column: 20, uri: "seseragi://artifact/monoid-wrappers", start: 1018, end: 1047 };
const $ssrg$source1030 = { file: "main.ssrg", line: 28, column: 32, uri: "seseragi://artifact/monoid-wrappers", start: 1030, end: 1047 };
const $ssrg$source1050 = { file: "main.ssrg", line: 29, column: 3, uri: "seseragi://artifact/monoid-wrappers", start: 1050, end: 1104 };
const $ssrg$source1060 = { file: "main.ssrg", line: 29, column: 13, uri: "seseragi://artifact/monoid-wrappers", start: 1060, end: 1104 };
const $ssrg$source1067 = { file: "main.ssrg", line: 29, column: 20, uri: "seseragi://artifact/monoid-wrappers", start: 1067, end: 1104 };
const $ssrg$source1083 = { file: "main.ssrg", line: 29, column: 36, uri: "seseragi://artifact/monoid-wrappers", start: 1083, end: 1104 };
const $ssrg$source1120 = { file: "main.ssrg", line: 30, column: 16, uri: "seseragi://artifact/monoid-wrappers", start: 1120, end: 1193 };
const $ssrg$source1132 = { file: "main.ssrg", line: 30, column: 28, uri: "seseragi://artifact/monoid-wrappers", start: 1132, end: 1193 };
const $ssrg$source1142 = { file: "main.ssrg", line: 30, column: 38, uri: "seseragi://artifact/monoid-wrappers", start: 1142, end: 1166 };
const $ssrg$source1168 = { file: "main.ssrg", line: 30, column: 64, uri: "seseragi://artifact/monoid-wrappers", start: 1168, end: 1192 };
const $ssrg$source1196 = { file: "main.ssrg", line: 31, column: 3, uri: "seseragi://artifact/monoid-wrappers", start: 1196, end: 1223 };
const $ssrg$source1206 = { file: "main.ssrg", line: 31, column: 13, uri: "seseragi://artifact/monoid-wrappers", start: 1206, end: 1223 };
const $ssrg$source1240 = { file: "main.ssrg", line: 32, column: 17, uri: "seseragi://artifact/monoid-wrappers", start: 1240, end: 1365 };
const $ssrg$source1260 = { file: "main.ssrg", line: 33, column: 5, uri: "seseragi://artifact/monoid-wrappers", start: 1260, end: 1365 };
const $ssrg$source1277 = { file: "main.ssrg", line: 34, column: 7, uri: "seseragi://artifact/monoid-wrappers", start: 1277, end: 1313 };
const $ssrg$source1321 = { file: "main.ssrg", line: 35, column: 7, uri: "seseragi://artifact/monoid-wrappers", start: 1321, end: 1359 };
const $ssrg$source1368 = { file: "main.ssrg", line: 37, column: 3, uri: "seseragi://artifact/monoid-wrappers", start: 1368, end: 1389 };
const $ssrg$source1428 = { file: "main.ssrg", line: 38, column: 39, uri: "seseragi://artifact/monoid-wrappers", start: 1428, end: 1443 };
const $ssrg$source1436 = { file: "main.ssrg", line: 38, column: 47, uri: "seseragi://artifact/monoid-wrappers", start: 1436, end: 1442 };
const $ssrg$source1446 = { file: "main.ssrg", line: 39, column: 3, uri: "seseragi://artifact/monoid-wrappers", start: 1446, end: 1491 };
const $ssrg$source1456 = { file: "main.ssrg", line: 39, column: 13, uri: "seseragi://artifact/monoid-wrappers", start: 1456, end: 1491 };
const $ssrg$source1463 = { file: "main.ssrg", line: 39, column: 20, uri: "seseragi://artifact/monoid-wrappers", start: 1463, end: 1491 };
const $ssrg$source1475 = { file: "main.ssrg", line: 39, column: 32, uri: "seseragi://artifact/monoid-wrappers", start: 1475, end: 1491 };
const $ssrg$source1484 = { file: "main.ssrg", line: 39, column: 41, uri: "seseragi://artifact/monoid-wrappers", start: 1484, end: 1490 };
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Tally: unique symbol;
export type Tally = {
  readonly "value": number;
  readonly [__ssrg$brand$Tally]: true;
};
declare const __ssrg$brand$Ordered: unique symbol;
export type Ordered = {
  readonly "value": string;
  readonly [__ssrg$brand$Ordered]: true;
};
export const __ssrg$instance$Zero$0 = { "zero": (unit: undefined) => (({ "value": _ssrg_int_subtract(0, 1, $ssrg$source102) } as const) as unknown as Tally) } as const;
export const __ssrg$instance$Add$1 = { "add": (left: Tally) => (right: Tally) => (({ "value": _ssrg_int_add(_ssrg_int_add((left)["value"], (right)["value"], $ssrg$source213), 1, $ssrg$source213) } as const) as unknown as Tally) } as const;
export const __ssrg$instance$One$2 = { "one": (unit: undefined) => (({ "value": "" } as const) as unknown as Ordered) } as const;
export const __ssrg$instance$Mul$3 = { "mul": (left: Ordered) => (right: Ordered) => (({ "value": $ssrg$atSource($ssrg$source483, () => _ssrg_string_add_dictionary["add"]((left)["value"])((right)["value"]), false) } as const) as unknown as Ordered) } as const;
export const mergeAll = <C, A,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => (__ssrg$evidence$1: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source586, () => _ssrg_collection_combine(__ssrg$evidence$0, __ssrg$evidence$1, values), false)
export const unwrapSum = <A,>(value: Sum<A>) => (($ssrg_match: Sum<A>): A => $ssrg_match.tag === "Sum" ? ((n: A): A => n)($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value)
export const unwrapProduct = <A,>(value: Product<A>) => (($ssrg_match: Product<A>): A => $ssrg_match.tag === "Product" ? ((n: A): A => n)($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source778, () => _ssrg_console_println($ssrg$atSource($ssrg$source788, () => ((_ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source795, () => unwrapSum<number>($ssrg$atSource($ssrg$source807, () => (((mergeAll as unknown as (argument: ReadonlyArray<Sum<number>>) => (argument: unknown) => (argument: unknown) => Sum<number>)([$ssrg$atSource($ssrg$source817, () => _ssrg_sum_Sum(1), false), $ssrg$atSource($ssrg$source824, () => _ssrg_sum_Sum(2), false), $ssrg$atSource($ssrg$source831, () => _ssrg_sum_Sum(3), false)])(_ssrg_array_reducible)(_ssrg_sum_monoid<number>(_ssrg_int_zero, _ssrg_int_add_dictionary))) as Sum<number>), false)), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source840, () => _ssrg_console_println($ssrg$atSource($ssrg$source850, () => ((_ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source857, () => unwrapProduct<number>($ssrg$atSource($ssrg$source873, () => (((mergeAll as unknown as (argument: List<Product<number>>) => (argument: unknown) => (argument: unknown) => Product<number>)(_ssrg_list_from_array([$ssrg$atSource($ssrg$source884, () => _ssrg_product_Product(2), false), $ssrg$atSource($ssrg$source895, () => _ssrg_product_Product(3), false), $ssrg$atSource($ssrg$source906, () => _ssrg_product_Product(4), false)]))(_ssrg_list_reducible)(_ssrg_product_monoid<number>(_ssrg_int_one, _ssrg_int_mul_dictionary))) as Product<number>), false)), false))) as string), false)), true), () => (() => { const emptySum: ReadonlyArray<Sum<number>> = [] as ReadonlyArray<Sum<number>>; return (() => { const emptyProduct: ReadonlyArray<Product<number>> = [] as ReadonlyArray<Product<number>>; return _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1001, () => _ssrg_console_println($ssrg$atSource($ssrg$source1011, () => ((_ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source1018, () => unwrapSum<number>($ssrg$atSource($ssrg$source1030, () => (((mergeAll as unknown as (argument: ReadonlyArray<Sum<number>>) => (argument: unknown) => (argument: unknown) => Sum<number>)(emptySum)(_ssrg_array_reducible)(_ssrg_sum_monoid<number>(_ssrg_int_zero, _ssrg_int_add_dictionary))) as Sum<number>), false)), false))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1050, () => _ssrg_console_println($ssrg$atSource($ssrg$source1060, () => ((_ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source1067, () => unwrapProduct<number>($ssrg$atSource($ssrg$source1083, () => (((mergeAll as unknown as (argument: ReadonlyArray<Product<number>>) => (argument: unknown) => (argument: unknown) => Product<number>)(emptyProduct)(_ssrg_array_reducible)(_ssrg_product_monoid<number>(_ssrg_int_one, _ssrg_int_mul_dictionary))) as Product<number>), false)), false))) as string), false)), true), () => (() => { const custom: Tally = $ssrg$atSource($ssrg$source1120, () => unwrapSum<Tally>($ssrg$atSource($ssrg$source1132, () => (((mergeAll as unknown as (argument: ReadonlyArray<Sum<Tally>>) => (argument: unknown) => (argument: unknown) => Sum<Tally>)([$ssrg$atSource($ssrg$source1142, () => _ssrg_sum_Sum((({ "value": 1 } as const) as unknown as Tally)), false), $ssrg$atSource($ssrg$source1168, () => _ssrg_sum_Sum((({ "value": 2 } as const) as unknown as Tally)), false)])(_ssrg_array_reducible)(_ssrg_sum_monoid<Tally>(__ssrg$instance$Zero$0, __ssrg$instance$Add$1))) as Sum<Tally>), false)), false); return _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1196, () => _ssrg_console_println($ssrg$atSource($ssrg$source1206, () => ((_ssrg_show_intShow["show"]((custom)["value"])) as string), false)), true), () => (() => { const ordered: Ordered = $ssrg$atSource($ssrg$source1240, () => unwrapProduct<Ordered>($ssrg$atSource($ssrg$source1260, () => (((mergeAll as unknown as (argument: ReadonlyArray<Product<Ordered>>) => (argument: unknown) => (argument: unknown) => Product<Ordered>)([$ssrg$atSource($ssrg$source1277, () => _ssrg_product_Product((({ "value": "first" } as const) as unknown as Ordered)), false), $ssrg$atSource($ssrg$source1321, () => _ssrg_product_Product((({ "value": "-second" } as const) as unknown as Ordered)), false)])(_ssrg_array_reducible)(_ssrg_product_monoid<Ordered>(__ssrg$instance$One$2, __ssrg$instance$Mul$3))) as Product<Ordered>), false)), false); return _ssrg_effect_flatMap($ssrg$atSource($ssrg$source1368, () => _ssrg_console_println((ordered)["value"]), true), () => (() => { const partial: (argument: Sum<number>) => Sum<number> = $ssrg$atSource($ssrg$source1428, () => ((_ssrg_sum_semigroup<number>(_ssrg_int_add_dictionary)["append"]($ssrg$atSource($ssrg$source1436, () => _ssrg_sum_Sum(10), false))) as (argument: Sum<number>) => Sum<number>), false); return $ssrg$atSource($ssrg$source1446, () => _ssrg_console_println($ssrg$atSource($ssrg$source1456, () => ((_ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source1463, () => unwrapSum<number>($ssrg$atSource($ssrg$source1475, () => partial($ssrg$atSource($ssrg$source1484, () => _ssrg_sum_Sum(20), false)), false)), false))) as string), false)), true); })()); })()); })())); })(); })()))

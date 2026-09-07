import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { subtract as _ssrg_int_subtract, add as _ssrg_int_add, intZero as _ssrg_int_zero, intAdd as _ssrg_int_add_dictionary, intOne as _ssrg_int_one, intMul as _ssrg_int_mul_dictionary } from "@seseragi/runtime/int"
import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { combine as _ssrg_collection_combine } from "@seseragi/runtime/collection"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { arrayReducible as _ssrg_array_reducible } from "@seseragi/runtime/array"
import { sumMonoid as _ssrg_sum_monoid, Sum as _ssrg_sum_Sum, productMonoid as _ssrg_product_monoid, Product as _ssrg_product_Product, sumSemigroup as _ssrg_sum_semigroup, type Sum as Sum, type Product as Product } from "@seseragi/runtime/sum"
import { listReducible as _ssrg_list_reducible, fromArray as _ssrg_list_from_array } from "@seseragi/runtime/list"
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
export const __ssrg$instance$Zero$0 = { "zero": (unit: undefined) => (({ "value": _ssrg_int_subtract(0, 1) } as const) as unknown as Tally) } as const;
export const __ssrg$instance$Add$1 = { "add": (left: Tally) => (right: Tally) => (({ "value": _ssrg_int_add(_ssrg_int_add((left)["value"], (right)["value"]), 1) } as const) as unknown as Tally) } as const;
export const __ssrg$instance$One$2 = { "one": (unit: undefined) => (({ "value": "" } as const) as unknown as Ordered) } as const;
export const __ssrg$instance$Mul$3 = { "mul": (left: Ordered) => (right: Ordered) => (({ "value": _ssrg_string_add_dictionary["add"]((left)["value"])((right)["value"]) } as const) as unknown as Ordered) } as const;
export const mergeAll = <C, A,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => (__ssrg$evidence$1: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_collection_combine(__ssrg$evidence$0, __ssrg$evidence$1, values)
export const unwrapSum = <A,>(value: Sum<A>) => (($ssrg_match: Sum<A>): A => $ssrg_match.tag === "Sum" ? ((n: A): A => n)($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(value)
export const unwrapProduct = <A,>(value: Product<A>) => (($ssrg_match: Product<A>): A => $ssrg_match.tag === "Product" ? ((n: A): A => n)($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_intShow["show"](unwrapSum(((mergeAll([_ssrg_sum_Sum(1), _ssrg_sum_Sum(2), _ssrg_sum_Sum(3)])(_ssrg_array_reducible)(_ssrg_sum_monoid<number>(_ssrg_int_zero, _ssrg_int_add_dictionary))) as Sum<number>)))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_intShow["show"](unwrapProduct(((mergeAll(_ssrg_list_from_array([_ssrg_product_Product(2), _ssrg_product_Product(3), _ssrg_product_Product(4)]))(_ssrg_list_reducible)(_ssrg_product_monoid<number>(_ssrg_int_one, _ssrg_int_mul_dictionary))) as Product<number>)))) as string)), () => (() => { const emptySum: ReadonlyArray<Sum<number>> = [] as ReadonlyArray<Sum<number>>; return (() => { const emptyProduct: ReadonlyArray<Product<number>> = [] as ReadonlyArray<Product<number>>; return _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_intShow["show"](unwrapSum(((mergeAll(emptySum)(_ssrg_array_reducible)(_ssrg_sum_monoid<number>(_ssrg_int_zero, _ssrg_int_add_dictionary))) as Sum<number>)))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_intShow["show"](unwrapProduct(((mergeAll(emptyProduct)(_ssrg_array_reducible)(_ssrg_product_monoid<number>(_ssrg_int_one, _ssrg_int_mul_dictionary))) as Product<number>)))) as string)), () => (() => { const custom: Tally = unwrapSum(((mergeAll([_ssrg_sum_Sum((({ "value": 1 } as const) as unknown as Tally)), _ssrg_sum_Sum((({ "value": 2 } as const) as unknown as Tally))])(_ssrg_array_reducible)(_ssrg_sum_monoid<Tally>(__ssrg$instance$Zero$0, __ssrg$instance$Add$1))) as Sum<Tally>)); return _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_intShow["show"]((custom)["value"])) as string)), () => (() => { const ordered: Ordered = unwrapProduct(((mergeAll([_ssrg_product_Product((({ "value": "first" } as const) as unknown as Ordered)), _ssrg_product_Product((({ "value": "-second" } as const) as unknown as Ordered))])(_ssrg_array_reducible)(_ssrg_product_monoid<Ordered>(__ssrg$instance$One$2, __ssrg$instance$Mul$3))) as Product<Ordered>)); return _ssrg_effect_flatMap(_ssrg_console_println((ordered)["value"]), () => (() => { const partial: (argument: Sum<number>) => Sum<number> = ((_ssrg_sum_semigroup<number>(_ssrg_int_add_dictionary)["append"](_ssrg_sum_Sum(10))) as (argument: Sum<number>) => Sum<number>); return _ssrg_console_println(((_ssrg_show_intShow["show"](unwrapSum(partial(_ssrg_sum_Sum(20))))) as string)); })()); })()); })())); })(); })()))

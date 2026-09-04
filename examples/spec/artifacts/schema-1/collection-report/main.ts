import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { intOrd as _ssrg_int_ord_dictionary } from "@seseragi/runtime/equality"
import { stringShow as _ssrg_show_stringShow, intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { arrayReducible as _ssrg_array_reducible, collectMap as _ssrg_array_comprehend, filter as _ssrg_array_filter, arrayFunctor as _ssrg_array_functor } from "@seseragi/runtime/array"
import { intZero as _ssrg_int_zero, intAdd as _ssrg_int_add_dictionary } from "@seseragi/runtime/int"
import { sum as _ssrg_collection_sum, join as _ssrg_collection_join } from "@seseragi/runtime/collection"
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Sale: unique symbol;
type Sale = {
  readonly "item": string;
  readonly "amount": number;
  readonly [__ssrg$brand$Sale]: true;
};
const completed = (sale: Sale) => (_ssrg_int_ord_dictionary["compare"]((sale)["amount"])(0))["tag"] === "Greater"
const renderSale = (sale: Sale) => _ssrg_show_stringShow["show"]((sale)["item"]) + ": " + _ssrg_show_intShow["show"]((sale)["amount"])
const totalSales = (values: ReadonlyArray<Sale>) => _ssrg_collection_sum(_ssrg_array_reducible, _ssrg_int_zero, _ssrg_int_add_dictionary, _ssrg_array_comprehend(values, (sale) => true, (sale) => (sale)["amount"]))
const report = (values: ReadonlyArray<Sale>) => (() => { const completedSales: ReadonlyArray<Sale> = _ssrg_array_filter(completed, values); return (() => { const lines: ReadonlyArray<string> = _ssrg_array_functor["map"](renderSale)(completedSales); return (() => { const total: number = totalSales(completedSales); return _ssrg_collection_join(_ssrg_array_reducible, "\n", ["Completed: " + _ssrg_show_stringShow["show"](_ssrg_collection_join(_ssrg_array_reducible, ", ", lines)), "Total: " + _ssrg_show_intShow["show"](total)]); })(); })(); })()
export const collectionReport = (unit: undefined) => report(sales)
const sales: ReadonlyArray<Sale> = [(({ "item": "Book", "amount": 2400 } as const) as unknown as Sale), (({ "item": "Pen", "amount": 600 } as const) as unknown as Sale), (({ "item": "Cancelled", "amount": 0 } as const) as unknown as Sale), (({ "item": "Notebook", "amount": 1200 } as const) as unknown as Sale)];

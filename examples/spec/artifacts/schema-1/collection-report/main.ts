import { stringShow as _ssrg_show_stringShow, intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { arrayReducible as _ssrg_array_reducible, collectMap as _ssrg_array_comprehend, filter as _ssrg_array_filter, arrayFunctor as _ssrg_array_functor } from "@seseragi/runtime/array"
import { intZero as _ssrg_int_zero, intAdd as _ssrg_int_add } from "@seseragi/runtime/int64"
import { sum as _ssrg_collection_sum, join as _ssrg_collection_join } from "@seseragi/runtime/collection"

declare const __ssrg$brand$Sale: unique symbol;
type Sale = {
  readonly "item": string;
  readonly "amount": bigint;
  readonly [__ssrg$brand$Sale]: true;
};
const sales: ReadonlyArray<Sale> = [(({ "item": "Book", "amount": 2400n } as const) as unknown as Sale), (({ "item": "Pen", "amount": 600n } as const) as unknown as Sale), (({ "item": "Cancelled", "amount": 0n } as const) as unknown as Sale), (({ "item": "Notebook", "amount": 1200n } as const) as unknown as Sale)];
const completed = (sale: Sale) => (sale)["amount"] > 0n
const renderSale = (sale: Sale) => _ssrg_show_stringShow["show"]((sale)["item"]) + ": " + _ssrg_show_intShow["show"]((sale)["amount"])
const totalSales = (values: ReadonlyArray<Sale>) => _ssrg_collection_sum(_ssrg_array_reducible, _ssrg_int_zero, _ssrg_int_add, _ssrg_array_comprehend(values, (sale) => true, (sale) => (sale)["amount"]))
const report = (values: ReadonlyArray<Sale>) => (() => { const completedSales: ReadonlyArray<Sale> = _ssrg_array_filter(completed, values); return (() => { const lines: ReadonlyArray<string> = _ssrg_array_functor["map"](renderSale)(completedSales); return (() => { const total: bigint = totalSales(completedSales); return _ssrg_collection_join(_ssrg_array_reducible, "\n", ["Completed: " + _ssrg_show_stringShow["show"](_ssrg_collection_join(_ssrg_array_reducible, ", ", lines)), "Total: " + _ssrg_show_intShow["show"](total)]); })(); })(); })()
export const collectionReport = (unit: undefined) => report(sales)

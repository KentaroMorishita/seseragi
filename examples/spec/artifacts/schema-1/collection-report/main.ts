import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intOrd as _ssrg_int_ord_dictionary } from "@seseragi/runtime/equality"
import { stringShow as _ssrg_show_stringShow, intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { arrayReducible as _ssrg_array_reducible, collectMap as _ssrg_array_comprehend, filter as _ssrg_array_filter, arrayFunctor as _ssrg_array_functor } from "@seseragi/runtime/array"
import { intZero as _ssrg_int_zero, intAdd as _ssrg_int_add_dictionary } from "@seseragi/runtime/int"
import { sum as _ssrg_collection_sum, join as _ssrg_collection_join } from "@seseragi/runtime/collection"
const $ssrg$source442 = { file: "main.ssrg", line: 22, column: 3, uri: "seseragi://artifact/collection-report", start: 442, end: 476 };
const $ssrg$source545 = { file: "main.ssrg", line: 25, column: 24, uri: "seseragi://artifact/collection-report", start: 545, end: 575 };
const $ssrg$source590 = { file: "main.ssrg", line: 26, column: 15, uri: "seseragi://artifact/collection-report", start: 590, end: 619 };
const $ssrg$source634 = { file: "main.ssrg", line: 27, column: 15, uri: "seseragi://artifact/collection-report", start: 634, end: 659 };
const $ssrg$source663 = { file: "main.ssrg", line: 29, column: 3, uri: "seseragi://artifact/collection-report", start: 663, end: 737 };
const $ssrg$source693 = { file: "main.ssrg", line: 30, column: 19, uri: "seseragi://artifact/collection-report", start: 693, end: 708 };
const $ssrg$source790 = { file: "main.ssrg", line: 36, column: 3, uri: "seseragi://artifact/collection-report", start: 790, end: 802 };
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Sale: unique symbol;
type Sale = {
  readonly "item": string;
  readonly "amount": number;
  readonly [__ssrg$brand$Sale]: true;
};
const completed = (sale: Sale) => (_ssrg_int_ord_dictionary["compare"]((sale)["amount"])(0))["tag"] === "Greater"
const renderSale = (sale: Sale) => _ssrg_show_stringShow["show"]((sale)["item"]) + ": " + _ssrg_show_intShow["show"]((sale)["amount"])
const totalSales = (values: ReadonlyArray<Sale>) => $ssrg$atSource($ssrg$source442, () => _ssrg_collection_sum(_ssrg_array_reducible, _ssrg_int_zero, _ssrg_int_add_dictionary, _ssrg_array_comprehend(values, (sale) => true, (sale) => (sale)["amount"])), false)
const report = (values: ReadonlyArray<Sale>) => (() => { const completedSales: ReadonlyArray<Sale> = $ssrg$atSource($ssrg$source545, () => _ssrg_array_filter(completed, values), false); return (() => { const lines: ReadonlyArray<string> = $ssrg$atSource($ssrg$source590, () => ((_ssrg_array_functor["map"](renderSale)(completedSales)) as ReadonlyArray<string>), false); return (() => { const total: number = $ssrg$atSource($ssrg$source634, () => totalSales(completedSales), false); return $ssrg$atSource($ssrg$source663, () => _ssrg_collection_join(_ssrg_array_reducible, "\n", ["Completed: " + _ssrg_show_stringShow["show"]($ssrg$atSource($ssrg$source693, () => _ssrg_collection_join(_ssrg_array_reducible, ", ", lines), false)), "Total: " + _ssrg_show_intShow["show"](total)]), false); })(); })(); })()
export const collectionReport = (unit: undefined) => $ssrg$atSource($ssrg$source790, () => report(sales), false)
const sales: ReadonlyArray<Sale> = [(({ "item": "Book", "amount": 2400 } as const) as unknown as Sale), (({ "item": "Pen", "amount": 600 } as const) as unknown as Sale), (({ "item": "Cancelled", "amount": 0 } as const) as unknown as Sale), (({ "item": "Notebook", "amount": 1200 } as const) as unknown as Sale)];

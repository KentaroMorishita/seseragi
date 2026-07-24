import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"

const sumArray = (values: ReadonlyArray<bigint>) => (() => { const loop = (total: bigint) => (rest: ReadonlyArray<bigint>) => (($ssrg_match: ReadonlyArray<bigint>): bigint => $ssrg_match.length === 0 ? total : ((head: bigint, tail: ReadonlyArray<bigint>): bigint => loop(_ssrg_int64_add(total, head))(tail))($ssrg_match[0], $ssrg_match.slice(1)))(rest); return loop(0n)(values); })()
const sumList = (values: List<bigint>) => (() => { const loop = (total: bigint) => (rest: List<bigint>) => (($ssrg_match: List<bigint>): bigint => $ssrg_match.tag === "Empty" ? total : ((head: bigint, tail: List<bigint>): bigint => loop(_ssrg_int64_add(total, head))(tail))($ssrg_match.head, $ssrg_match.tail))(rest); return loop(0n)(values); })()
export const collectionRestWorks = (unit: undefined) => (() => { const arrayTotal: bigint = sumArray([1n, 2n, 3n, 4n]); return (() => { const listTotal: bigint = sumList(_ssrg_list_from_array([1n, 2n, 3n, 4n])); return arrayTotal === 10n ? listTotal === 10n : false; })(); })()

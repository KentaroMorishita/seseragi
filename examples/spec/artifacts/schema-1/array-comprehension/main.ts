import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { collectFlatMap as _ssrg_array_comprehend_flat, collectMap as _ssrg_array_comprehend } from "@seseragi/runtime/array"

export const selectedPairs = (unit: undefined) => _ssrg_array_comprehend_flat([1n, 2n], (left) => true, (left) => _ssrg_array_comprehend([10n, 20n], (right) => right === 20n, (right) => _ssrg_int64_add(left, right)))

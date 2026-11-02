import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { combine as _ssrg_collection_combine, join as _ssrg_collection_join } from "@seseragi/runtime/collection"
import { arrayReducible as _ssrg_array_reducible, arrayMonoid as _ssrg_array_monoid, arraySemigroup as _ssrg_array_semigroup } from "@seseragi/runtime/array"
import { stringMonoid as _ssrg_string_monoid, stringSemigroup as _ssrg_string_semigroup } from "@seseragi/runtime/string"
import { listMonoid as _ssrg_list_monoid, fromArray as _ssrg_list_from_array, listReducible as _ssrg_list_reducible, listSemigroup as _ssrg_list_semigroup, type List as List } from "@seseragi/runtime/list"
const $ssrg$source75 = { file: "main.ssrg", line: 3, column: 3, uri: "seseragi://artifact/collection-combine", start: 75, end: 89 };
const $ssrg$source134 = { file: "main.ssrg", line: 6, column: 3, uri: "seseragi://artifact/collection-combine", start: 134, end: 164 };
const $ssrg$source221 = { file: "main.ssrg", line: 9, column: 3, uri: "seseragi://artifact/collection-combine", start: 221, end: 254 };
const $ssrg$source298 = { file: "main.ssrg", line: 12, column: 3, uri: "seseragi://artifact/collection-combine", start: 298, end: 331 };
const $ssrg$source308 = { file: "main.ssrg", line: 12, column: 13, uri: "seseragi://artifact/collection-combine", start: 308, end: 330 };
const $ssrg$source386 = { file: "main.ssrg", line: 15, column: 3, uri: "seseragi://artifact/collection-combine", start: 386, end: 424 };
const $ssrg$source467 = { file: "main.ssrg", line: 18, column: 3, uri: "seseragi://artifact/collection-combine", start: 467, end: 499 };
const $ssrg$source477 = { file: "main.ssrg", line: 18, column: 13, uri: "seseragi://artifact/collection-combine", start: 477, end: 498 };
const $ssrg$source595 = { file: "main.ssrg", line: 24, column: 3, uri: "seseragi://artifact/collection-combine", start: 595, end: 626 };
const $ssrg$source610 = { file: "main.ssrg", line: 24, column: 18, uri: "seseragi://artifact/collection-combine", start: 610, end: 625 };
const $ssrg$source696 = { file: "main.ssrg", line: 28, column: 3, uri: "seseragi://artifact/collection-combine", start: 696, end: 713 };
const $ssrg$source765 = { file: "main.ssrg", line: 31, column: 3, uri: "seseragi://artifact/collection-combine", start: 765, end: 797 };
const $ssrg$source842 = { file: "main.ssrg", line: 34, column: 3, uri: "seseragi://artifact/collection-combine", start: 842, end: 863 };
const $ssrg$source920 = { file: "main.ssrg", line: 37, column: 3, uri: "seseragi://artifact/collection-combine", start: 920, end: 947 };
const $ssrg$source991 = { file: "main.ssrg", line: 40, column: 3, uri: "seseragi://artifact/collection-combine", start: 991, end: 1024 };
const $ssrg$source1001 = { file: "main.ssrg", line: 40, column: 13, uri: "seseragi://artifact/collection-combine", start: 1001, end: 1023 };
const $ssrg$source1079 = { file: "main.ssrg", line: 43, column: 3, uri: "seseragi://artifact/collection-combine", start: 1079, end: 1107 };
const $ssrg$source1150 = { file: "main.ssrg", line: 46, column: 3, uri: "seseragi://artifact/collection-combine", start: 1150, end: 1182 };
const $ssrg$source1160 = { file: "main.ssrg", line: 46, column: 13, uri: "seseragi://artifact/collection-combine", start: 1160, end: 1181 };
const $ssrg$source1254 = { file: "main.ssrg", line: 50, column: 5, uri: "seseragi://artifact/collection-combine", start: 1254, end: 1271 };
const $ssrg$source1277 = { file: "main.ssrg", line: 51, column: 5, uri: "seseragi://artifact/collection-combine", start: 1277, end: 1293 };
const $ssrg$source1299 = { file: "main.ssrg", line: 52, column: 5, uri: "seseragi://artifact/collection-combine", start: 1299, end: 1314 };
const $ssrg$source1320 = { file: "main.ssrg", line: 53, column: 5, uri: "seseragi://artifact/collection-combine", start: 1320, end: 1336 };
const $ssrg$source1342 = { file: "main.ssrg", line: 54, column: 5, uri: "seseragi://artifact/collection-combine", start: 1342, end: 1366 };
const $ssrg$source1372 = { file: "main.ssrg", line: 55, column: 5, uri: "seseragi://artifact/collection-combine", start: 1372, end: 1389 };
const $ssrg$source1395 = { file: "main.ssrg", line: 56, column: 5, uri: "seseragi://artifact/collection-combine", start: 1395, end: 1411 };
const $ssrg$source1417 = { file: "main.ssrg", line: 57, column: 5, uri: "seseragi://artifact/collection-combine", start: 1417, end: 1432 };
$ssrg$assertUnicodeVersion("17.0.0")

const combineValues = <C, A,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => (__ssrg$evidence$1: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source75, () => _ssrg_collection_combine(__ssrg$evidence$0, __ssrg$evidence$1, values), false)
const combinedString = (unit: undefined) => $ssrg$atSource($ssrg$source134, () => ((combineValues<ReadonlyArray<string>, string>(["Sese", "ragi"])(_ssrg_array_reducible)(_ssrg_string_monoid)) as string), false)
const combinedArrayValues = (unit: undefined) => $ssrg$atSource($ssrg$source221, () => ((combineValues<ReadonlyArray<ReadonlyArray<string>>, ReadonlyArray<string>>([["a", "b"], ["c"]])(_ssrg_array_reducible)(_ssrg_array_monoid)) as ReadonlyArray<string>), false)
const combinedArray = (unit: undefined) => $ssrg$atSource($ssrg$source298, () => _ssrg_collection_join(_ssrg_array_reducible, "|", $ssrg$atSource($ssrg$source308, () => combinedArrayValues(undefined), false)), false)
const combinedListValues = (unit: undefined) => $ssrg$atSource($ssrg$source386, () => ((combineValues<ReadonlyArray<List<string>>, List<string>>([_ssrg_list_from_array(["l1", "l2"]), _ssrg_list_from_array(["l3"])])(_ssrg_array_reducible)(_ssrg_list_monoid)) as List<string>), false)
const combinedList = (unit: undefined) => $ssrg$atSource($ssrg$source467, () => _ssrg_collection_join(_ssrg_list_reducible, "|", $ssrg$atSource($ssrg$source477, () => combinedListValues(undefined), false)), false)
const emptyStrings = (unit: undefined) => [] as ReadonlyArray<string>
const combinedEmpty = (unit: undefined) => $ssrg$atSource($ssrg$source595, () => ((combineValues<ReadonlyArray<string>, string>($ssrg$atSource($ssrg$source610, () => emptyStrings(undefined), false))(_ssrg_array_reducible)(_ssrg_string_monoid)) as string), false)
const appendWithMonoid = <A,>(left: A) => (right: A) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source696, () => ((__ssrg$evidence$0["append"](left)(right)) as A), false)
const combinedViaSupertrait = (unit: undefined) => $ssrg$atSource($ssrg$source765, () => ((appendWithMonoid<string>("super")("trait")(_ssrg_string_monoid)) as string), false)
const appendedString = (unit: undefined) => $ssrg$atSource($ssrg$source842, () => ((_ssrg_string_semigroup["append"]("semi")("group")) as string), false)
const appendedArrayValues = (unit: undefined) => $ssrg$atSource($ssrg$source920, () => ((_ssrg_array_semigroup["append"](["array"])(["append"])) as ReadonlyArray<string>), false)
const appendedArray = (unit: undefined) => $ssrg$atSource($ssrg$source991, () => _ssrg_collection_join(_ssrg_array_reducible, "|", $ssrg$atSource($ssrg$source1001, () => appendedArrayValues(undefined), false)), false)
const appendedListValues = (unit: undefined) => $ssrg$atSource($ssrg$source1079, () => ((_ssrg_list_semigroup["append"](_ssrg_list_from_array(["list"]))(_ssrg_list_from_array(["append"]))) as List<string>), false)
const appendedList = (unit: undefined) => $ssrg$atSource($ssrg$source1150, () => _ssrg_collection_join(_ssrg_list_reducible, "|", $ssrg$atSource($ssrg$source1160, () => appendedListValues(undefined), false)), false)
export const collectionCombineResults = (unit: undefined) => [$ssrg$atSource($ssrg$source1254, () => combinedString(undefined), false), $ssrg$atSource($ssrg$source1277, () => combinedArray(undefined), false), $ssrg$atSource($ssrg$source1299, () => combinedList(undefined), false), $ssrg$atSource($ssrg$source1320, () => combinedEmpty(undefined), false), $ssrg$atSource($ssrg$source1342, () => combinedViaSupertrait(undefined), false), $ssrg$atSource($ssrg$source1372, () => appendedString(undefined), false), $ssrg$atSource($ssrg$source1395, () => appendedArray(undefined), false), $ssrg$atSource($ssrg$source1417, () => appendedList(undefined), false)]

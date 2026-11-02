import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"
import { arrayReducible as _ssrg_array_reducible } from "@seseragi/runtime/array"
import { listReducible as _ssrg_list_reducible, fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"
const $ssrg$source89 = { file: "main.ssrg", line: 3, column: 3, uri: "seseragi://artifact/collection-join", start: 89, end: 110 };
const $ssrg$source162 = { file: "main.ssrg", line: 6, column: 3, uri: "seseragi://artifact/collection-join", start: 162, end: 185 };
const $ssrg$source235 = { file: "main.ssrg", line: 9, column: 3, uri: "seseragi://artifact/collection-join", start: 235, end: 258 };
const $ssrg$source310 = { file: "main.ssrg", line: 12, column: 3, uri: "seseragi://artifact/collection-join", start: 310, end: 332 };
const $ssrg$source401 = { file: "main.ssrg", line: 16, column: 5, uri: "seseragi://artifact/collection-join", start: 401, end: 438 };
const $ssrg$source444 = { file: "main.ssrg", line: 17, column: 5, uri: "seseragi://artifact/collection-join", start: 444, end: 472 };
const $ssrg$source478 = { file: "main.ssrg", line: 18, column: 5, uri: "seseragi://artifact/collection-join", start: 478, end: 491 };
$ssrg$assertUnicodeVersion("17.0.0")

const joinValues = <C,>(separator: string) => (values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source89, () => _ssrg_collection_join(__ssrg$evidence$0, separator, values), false)
const arrayLabel = (values: ReadonlyArray<string>) => $ssrg$atSource($ssrg$source162, () => ((joinValues<ReadonlyArray<string>>(" / ")(values)(_ssrg_array_reducible)) as string), false)
const listLabel = (values: List<string>) => $ssrg$atSource($ssrg$source235, () => ((joinValues<List<string>>(" + ")(values)(_ssrg_list_reducible)) as string), false)
const emptyLabel = (values: ReadonlyArray<string>) => $ssrg$atSource($ssrg$source310, () => ((joinValues<ReadonlyArray<string>>(", ")(values)(_ssrg_array_reducible)) as string), false)
export const collectionJoinResults = (unit: undefined) => [$ssrg$atSource($ssrg$source401, () => arrayLabel(["alpha", "beta", "gamma"]), false), $ssrg$atSource($ssrg$source444, () => listLabel(_ssrg_list_from_array(["left", "right"])), false), $ssrg$atSource($ssrg$source478, () => emptyLabel([] as ReadonlyArray<string>), false)]

import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"
import { arrayReducible as _ssrg_array_reducible } from "@seseragi/runtime/array"
import { listReducible as _ssrg_list_reducible, fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"

const joinValues = <C,>(separator: string) => (values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_collection_join(__ssrg$evidence$0, separator, values)
const arrayLabel = (values: ReadonlyArray<string>) => joinValues(" / ")(values)(_ssrg_array_reducible)
const listLabel = (values: List<string>) => joinValues(" + ")(values)(_ssrg_list_reducible)
const emptyLabel = (values: ReadonlyArray<string>) => joinValues(", ")(values)(_ssrg_array_reducible)
export const collectionJoinResults = (unit: undefined) => [arrayLabel(["alpha", "beta", "gamma"]), listLabel(_ssrg_list_from_array(["left", "right"])), emptyLabel([] as ReadonlyArray<string>)]

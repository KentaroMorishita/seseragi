import type { Pair, Query, Row } from "./facade"
import { through } from "./main"

const read: Query<number> = (value) => value + 1
const fields: Row<number> = { value: 7, read }
const pair: Pair<number> = [9, 10]
const result: number = through(read)
// @ts-expect-error A Query has an Int parameter, even after renamed reexports.
const wrongParameter: Query<number> = (_value: boolean) => 1
// @ts-expect-error A Query keeps its selected result type.
const wrongResult: Query<number> = (_value: number) => "wrong"
// @ts-expect-error A Row keeps the field's selected value type.
const wrongField: Row<number> = { value: "wrong", read }
// @ts-expect-error A Pair retains both selected element types.
const wrongPair: Pair<number> = [9, "wrong"]
void [fields, pair, result, wrongParameter, wrongResult, wrongField, wrongPair]

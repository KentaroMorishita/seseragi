import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { append as _ssrg_array_append, filter as _ssrg_array_filter, arrayReducible as _ssrg_array_reducible, arrayFunctor as _ssrg_array_functor } from "@seseragi/runtime/array"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { intShow as _ssrg_show_intShow, stringShow as _ssrg_show_stringShow } from "@seseragi/runtime/show"
import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"
const $ssrg$source307 = { file: "main.ssrg", line: 15, column: 3, uri: "seseragi://artifact/todo-workflow", start: 307, end: 334 };
const $ssrg$source475 = { file: "main.ssrg", line: 21, column: 3, uri: "seseragi://artifact/todo-workflow", start: 475, end: 509 };
const $ssrg$source490 = { file: "main.ssrg", line: 21, column: 18, uri: "seseragi://artifact/todo-workflow", start: 490, end: 501 };
const $ssrg$source613 = { file: "main.ssrg", line: 27, column: 3, uri: "seseragi://artifact/todo-workflow", start: 613, end: 642 };
const $ssrg$source761 = { file: "main.ssrg", line: 33, column: 3, uri: "seseragi://artifact/todo-workflow", start: 761, end: 794 };
const $ssrg$source773 = { file: "main.ssrg", line: 33, column: 15, uri: "seseragi://artifact/todo-workflow", start: 773, end: 794 };
const $ssrg$source865 = { file: "main.ssrg", line: 36, column: 18, uri: "seseragi://artifact/todo-workflow", start: 865, end: 938 };
const $ssrg$source959 = { file: "main.ssrg", line: 37, column: 21, uri: "seseragi://artifact/todo-workflow", start: 959, end: 980 };
const $ssrg$source1000 = { file: "main.ssrg", line: 38, column: 20, uri: "seseragi://artifact/todo-workflow", start: 1000, end: 1023 };
const $ssrg$source1047 = { file: "main.ssrg", line: 41, column: 19, uri: "seseragi://artifact/todo-workflow", start: 1047, end: 1067 };
const $ssrg$source1092 = { file: "main.ssrg", line: 42, column: 22, uri: "seseragi://artifact/todo-workflow", start: 1092, end: 1115 };
const $ssrg$source1134 = { file: "main.ssrg", line: 43, column: 16, uri: "seseragi://artifact/todo-workflow", start: 1134, end: 1156 };
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Todo: unique symbol;
type Todo = {
  readonly "id": number;
  readonly "title": string;
  readonly "urgent": boolean;
  readonly [__ssrg$brand$Todo]: true;
};
const addTodo = (todo: Todo) => (values: ReadonlyArray<Todo>) => $ssrg$atSource($ssrg$source307, () => _ssrg_array_append<Todo>([todo], values), false)
const keepTodo = (removedId: number) => (todo: Todo) => _ssrg_int_eq_dictionary["eq"]((todo)["id"])(removedId) === false
const removeTodo = (id: number) => (values: ReadonlyArray<Todo>) => $ssrg$atSource($ssrg$source475, () => _ssrg_array_filter($ssrg$atSource($ssrg$source490, () => keepTodo(id), false), values), false)
const isUrgent = (todo: Todo) => (todo)["urgent"]
const urgentTodos = (values: ReadonlyArray<Todo>) => $ssrg$atSource($ssrg$source613, () => _ssrg_array_filter(isUrgent, values), false)
const renderTodo = (todo: Todo) => _ssrg_show_intShow["show"]((todo)["id"]) + ": " + _ssrg_show_stringShow["show"]((todo)["title"])
const renderTodos = (values: ReadonlyArray<Todo>) => $ssrg$atSource($ssrg$source761, () => _ssrg_collection_join(_ssrg_array_reducible, ", ", $ssrg$atSource($ssrg$source773, () => ((_ssrg_array_functor["map"](renderTodo)(values)) as ReadonlyArray<string>), false)), false)
export const todoWorkflow = (unit: undefined) => (() => { const afterAdd: ReadonlyArray<Todo> = $ssrg$atSource($ssrg$source865, () => addTodo((({ "id": 3, "title": "Review docs", "urgent": true } as const) as unknown as Todo))(initialTodos), false); return (() => { const afterRemove: ReadonlyArray<Todo> = $ssrg$atSource($ssrg$source959, () => removeTodo(2)(afterAdd), false); return (() => { const urgentOnly: ReadonlyArray<Todo> = $ssrg$atSource($ssrg$source1000, () => urgentTodos(afterRemove), false); return ["after add: " + _ssrg_show_stringShow["show"]($ssrg$atSource($ssrg$source1047, () => renderTodos(afterAdd), false)), "after delete: " + _ssrg_show_stringShow["show"]($ssrg$atSource($ssrg$source1092, () => renderTodos(afterRemove), false)), "urgent: " + _ssrg_show_stringShow["show"]($ssrg$atSource($ssrg$source1134, () => renderTodos(urgentOnly), false))]; })(); })(); })()
const initialTodos: ReadonlyArray<Todo> = [(({ "id": 1, "title": "Write notes", "urgent": false } as const) as unknown as Todo), (({ "id": 2, "title": "Ship release", "urgent": true } as const) as unknown as Todo)];

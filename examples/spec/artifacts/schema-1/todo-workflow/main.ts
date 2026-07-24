import { append as _ssrg_array_append, filter as _ssrg_array_filter, arrayReducible as _ssrg_array_reducible, arrayFunctor as _ssrg_array_functor } from "@seseragi/runtime/array"
import { intShow as _ssrg_show_intShow, stringShow as _ssrg_show_stringShow } from "@seseragi/runtime/show"
import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"

declare const __ssrg$brand$Todo: unique symbol;
type Todo = {
  readonly "id": bigint;
  readonly "title": string;
  readonly "urgent": boolean;
  readonly [__ssrg$brand$Todo]: true;
};
const initialTodos: ReadonlyArray<Todo> = [(({ "id": 1n, "title": "Write notes", "urgent": false } as const) as unknown as Todo), (({ "id": 2n, "title": "Ship release", "urgent": true } as const) as unknown as Todo)];
const addTodo = (todo: Todo) => (values: ReadonlyArray<Todo>) => _ssrg_array_append([todo], values)
const keepTodo = (removedId: bigint) => (todo: Todo) => (todo)["id"] !== removedId
const removeTodo = (id: bigint) => (values: ReadonlyArray<Todo>) => _ssrg_array_filter(keepTodo(id), values)
const isUrgent = (todo: Todo) => (todo)["urgent"]
const urgentTodos = (values: ReadonlyArray<Todo>) => _ssrg_array_filter(isUrgent, values)
const renderTodo = (todo: Todo) => _ssrg_show_intShow["show"]((todo)["id"]) + ": " + _ssrg_show_stringShow["show"]((todo)["title"])
const renderTodos = (values: ReadonlyArray<Todo>) => _ssrg_collection_join(_ssrg_array_reducible, ", ", _ssrg_array_functor["map"](renderTodo)(values))
export const todoWorkflow = (unit: undefined) => (() => { const afterAdd: ReadonlyArray<Todo> = addTodo((({ "id": 3n, "title": "Review docs", "urgent": true } as const) as unknown as Todo))(initialTodos); return (() => { const afterRemove: ReadonlyArray<Todo> = removeTodo(2n)(afterAdd); return (() => { const urgentOnly: ReadonlyArray<Todo> = urgentTodos(afterRemove); return ["after add: " + _ssrg_show_stringShow["show"](renderTodos(afterAdd)), "after delete: " + _ssrg_show_stringShow["show"](renderTodos(afterRemove)), "urgent: " + _ssrg_show_stringShow["show"](renderTodos(urgentOnly))]; })(); })(); })()

import { view as todoForm } from "./form.js"
import { view as todoList } from "./list.js"
import { append as _ssrg_array_append, arrayReducible as _ssrg_array_reducible } from "@seseragi/runtime/array"
import { update as _ssrg_signal_update, make as _ssrg_signal_make, map as _ssrg_signal_map, type MutableSignal as MutableSignal, type Signal as Signal } from "@seseragi/runtime/signal"
import { join as _ssrg_collection_join } from "@seseragi/runtime/collection"
import { section as _ssrg_html_section, h2 as _ssrg_html_h2, type InputEvent as InputEvent, type Html as Html } from "@seseragi/runtime/html"
import { flatMap as _ssrg_effect_flatMap, succeed as _ssrg_effect_succeed, type Effect as Effect } from "@seseragi/runtime/effect"

type TodoAction =
  | { readonly tag: "DraftChanged"; readonly value: string }
  | { readonly tag: "Submitted" }
  | { readonly tag: "Cleared" };
const DraftChanged = (value: string): TodoAction => ({ tag: "DraftChanged", value } as const);
const Submitted: TodoAction = { tag: "Submitted" } as const;
const Cleared: TodoAction = { tag: "Cleared" } as const;
declare const __ssrg$brand$TodoState: unique symbol;
type TodoState = {
  readonly "draft": string;
  readonly "items": ReadonlyArray<string>;
  readonly [__ssrg$brand$TodoState]: true;
};
const update = (action: TodoAction) => (state: TodoState) => (($ssrg_match: TodoAction): TodoState => $ssrg_match.tag === "DraftChanged" ? ((value: string): TodoState => (({ ...state, "draft": value } as const) as unknown as TodoState))($ssrg_match.value) : $ssrg_match.tag === "Submitted" ? (state)["draft"] === "" ? state : (({ "draft": "", "items": _ssrg_array_append([(state)["draft"]], (state)["items"]) } as const) as unknown as TodoState) : (({ ...state, "items": [] as ReadonlyArray<string> } as const) as unknown as TodoState))(action)
const dispatch = (state: MutableSignal<TodoState>) => (action: TodoAction) => _ssrg_signal_update(update(action), state)
const draftAction = (event: InputEvent) => DraftChanged((event)["value"])
const formView = (state: MutableSignal<TodoState>) => (current: TodoState) => todoForm((current)["draft"])((event: InputEvent) => dispatch(state)(draftAction(event)))(dispatch(state)(Submitted))
const listView = (state: MutableSignal<TodoState>) => (current: TodoState) => todoList(_ssrg_collection_join(_ssrg_array_reducible, ", ", (current)["items"]))(dispatch(state)(Cleared))
const view = (state: MutableSignal<TodoState>) => (current: TodoState) => _ssrg_html_section(({ "children": [formView(state)(current), listView(state)(current), _ssrg_html_h2(({ "children": "Shared Todo" } as const))] } as const))
export const create = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_signal_make<TodoState>((({ "draft": "", "items": [] as ReadonlyArray<string> } as const) as unknown as TodoState)), (state: MutableSignal<TodoState>) => _ssrg_effect_succeed(_ssrg_signal_map(view(state), state)))

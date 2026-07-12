import { stringShow as _ssrg_show_stringShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { fail as _ssrg_effect_fail } from "@seseragi/runtime/effect"

export type InputError =
  | { readonly tag: "InvalidInput"; readonly value: string };
export const InvalidInput = (value: string): InputError => ({ tag: "InvalidInput", value } as const);
export const __ssrg$instance$Show$0: _ssrg_show_Show<InputError> = { show: (value: InputError): string => { switch (value.tag) { case "InvalidInput": return "InvalidInput" + " " + _ssrg_show_stringShow.show(value.value); } } };
export const reject = (input: string) => _ssrg_effect_fail(InvalidInput(input))

import { type InputError, reject, __ssrg$instance$Show$0 as __ssrg$instance$Show$0_1 } from "./input.js"
import { mapError as _ssrg_effect_mapError } from "@seseragi/runtime/effect"
import { type Show as _ssrg_show_Show } from "@seseragi/runtime/show"

export type AppError =
  | { readonly tag: "InputFailure"; readonly value: InputError };
export const InputFailure = (value: InputError): AppError => ({ tag: "InputFailure", value } as const);
export const __ssrg$instance$Show$0: _ssrg_show_Show<AppError> = { show: (value: AppError): string => { switch (value.tag) { case "InputFailure": return "InputFailure" + " " + __ssrg$instance$Show$0_1.show(value.value); } } };
export const main = (_unit: undefined) => _ssrg_effect_mapError(InputFailure, reject("lizard"))

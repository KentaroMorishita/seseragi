import { decide, renderOutcome, type Hand, type Outcome } from "./domain.js"
import { type InputError, readHand, __ssrg$instance$Show$0 as __ssrg$instance$Show$0_1 } from "./input.js"
import { consoleErrorShow as _ssrg_show_consoleErrorShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println, type ConsoleError as ConsoleError } from "@seseragi/runtime/console"

export type AppError =
  | { readonly tag: "InputFailure"; readonly value: InputError }
  | { readonly tag: "ConsoleFailure"; readonly value: ConsoleError };
export const InputFailure = (value: InputError): AppError => ({ tag: "InputFailure", value } as const);
export const ConsoleFailure = (value: ConsoleError): AppError => ({ tag: "ConsoleFailure", value } as const);
export const __ssrg$instance$Show$0: _ssrg_show_Show<AppError> = { show: (value: AppError): string => { switch (value.tag) { case "InputFailure": return "InputFailure" + " " + __ssrg$instance$Show$0_1.show(value.value); case "ConsoleFailure": return "ConsoleFailure" + " " + _ssrg_show_consoleErrorShow.show(value.value); } } };
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_effect_mapError(InputFailure, readHand(undefined)), (first: Hand) => _ssrg_effect_flatMap(_ssrg_effect_mapError(InputFailure, readHand(undefined)), (second: Hand) => (() => { const outcome: Outcome = decide(first)(second); return _ssrg_effect_mapError(ConsoleFailure, _ssrg_console_println(renderOutcome(outcome))); })()))

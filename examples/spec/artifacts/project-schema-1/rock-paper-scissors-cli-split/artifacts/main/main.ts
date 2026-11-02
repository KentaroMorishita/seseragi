import { decide, renderOutcome, type Hand, type Outcome } from "./domain.js"
import { type InputError, readHand, __ssrg$instance$Show$0 as __ssrg$instance$Show$0_1 } from "./input.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { boundedShow as _ssrg_show_boundedShow, consoleErrorShow as _ssrg_show_consoleErrorShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap, mapError as _ssrg_effect_mapError } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println, type ConsoleError as ConsoleError } from "@seseragi/runtime/console"
const $ssrg$source233 = { file: "project/rock-paper-scissors-cli-split/src/main.ssrg", line: 10, column: 14, uri: "seseragi://fixture/rps-cli-split::main", start: 233, end: 268 };
const $ssrg$source256 = { file: "project/rock-paper-scissors-cli-split/src/main.ssrg", line: 10, column: 37, uri: "seseragi://fixture/rps-cli-split::main", start: 256, end: 267 };
const $ssrg$source283 = { file: "project/rock-paper-scissors-cli-split/src/main.ssrg", line: 11, column: 15, uri: "seseragi://fixture/rps-cli-split::main", start: 283, end: 318 };
const $ssrg$source306 = { file: "project/rock-paper-scissors-cli-split/src/main.ssrg", line: 11, column: 38, uri: "seseragi://fixture/rps-cli-split::main", start: 306, end: 317 };
const $ssrg$source337 = { file: "project/rock-paper-scissors-cli-split/src/main.ssrg", line: 12, column: 19, uri: "seseragi://fixture/rps-cli-split::main", start: 337, end: 356 };
const $ssrg$source361 = { file: "project/rock-paper-scissors-cli-split/src/main.ssrg", line: 13, column: 5, uri: "seseragi://fixture/rps-cli-split::main", start: 361, end: 418 };
const $ssrg$source386 = { file: "project/rock-paper-scissors-cli-split/src/main.ssrg", line: 13, column: 30, uri: "seseragi://fixture/rps-cli-split::main", start: 386, end: 417 };
const $ssrg$source395 = { file: "project/rock-paper-scissors-cli-split/src/main.ssrg", line: 13, column: 39, uri: "seseragi://fixture/rps-cli-split::main", start: 395, end: 416 };
$ssrg$assertUnicodeVersion("17.0.0")

export type AppError =
  | { readonly tag: "InputFailure"; readonly value: InputError }
  | { readonly tag: "ConsoleFailure"; readonly value: ConsoleError };
export const InputFailure = (value: InputError): AppError => ({ tag: "InputFailure", value } as const);
export const ConsoleFailure = (value: ConsoleError): AppError => ({ tag: "ConsoleFailure", value } as const);
export const __ssrg$instance$Show$0: _ssrg_show_Show<AppError> = _ssrg_show_boundedShow((value: AppError): string => { switch (value.tag) { case "InputFailure": return "InputFailure" + " " + __ssrg$instance$Show$0_1.show(value.value); case "ConsoleFailure": return "ConsoleFailure" + " " + _ssrg_show_consoleErrorShow.show(value.value); } });
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source233, () => _ssrg_effect_mapError(InputFailure, $ssrg$atSource($ssrg$source256, () => readHand(undefined), true)), true), (first: Hand) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source283, () => _ssrg_effect_mapError(InputFailure, $ssrg$atSource($ssrg$source306, () => readHand(undefined), true)), true), (second: Hand) => (() => { const outcome: Outcome = $ssrg$atSource($ssrg$source337, () => decide(first)(second), false); return $ssrg$atSource($ssrg$source361, () => _ssrg_effect_mapError(ConsoleFailure, $ssrg$atSource($ssrg$source386, () => _ssrg_console_println($ssrg$atSource($ssrg$source395, () => renderOutcome(outcome), false)), true)), true); })()))

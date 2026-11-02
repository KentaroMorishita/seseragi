import { rejectViaFacade } from "./facade.js"
import { type InputError } from "./provider.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { flatMap as _ssrg_effect_flatMap, succeed as _ssrg_effect_succeed } from "@seseragi/runtime/effect"
const $ssrg$source76 = { file: "project/transitive-effect-failure/src/main.ssrg", line: 5, column: 5, uri: "seseragi://fixture/transitive-effect-failure::main", start: 76, end: 100 };
const $ssrg$source105 = { file: "project/transitive-effect-failure/src/main.ssrg", line: 6, column: 5, uri: "seseragi://fixture/transitive-effect-failure::main", start: 105, end: 115 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source76, () => rejectViaFacade("lizard"), true), () => $ssrg$atSource($ssrg$source105, () => _ssrg_effect_succeed(undefined), true))

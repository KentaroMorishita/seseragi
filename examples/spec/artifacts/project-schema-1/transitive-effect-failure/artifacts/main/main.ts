import { rejectViaFacade } from "./facade.js"
import { type InputError } from "./provider.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { flatMap as _ssrg_effect_flatMap, succeed as _ssrg_effect_succeed } from "@seseragi/runtime/effect"
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => _ssrg_effect_flatMap(rejectViaFacade("lizard"), () => _ssrg_effect_succeed(undefined))

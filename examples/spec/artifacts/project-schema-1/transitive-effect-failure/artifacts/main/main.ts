import { rejectViaFacade } from "./facade.js"
import { type InputError } from "./provider.js"
import { flatMap as _ssrg_effect_flatMap, succeed as _ssrg_effect_succeed } from "@seseragi/runtime/effect"

export const main = (_unit: undefined) => _ssrg_effect_flatMap(rejectViaFacade("lizard"), () => _ssrg_effect_succeed(undefined))

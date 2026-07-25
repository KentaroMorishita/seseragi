import { duplicate } from "./domain.js"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const swap = <A,>(value: { readonly "left": A; readonly "right": A }) => ({ "left": (value)["right"], "right": (value)["left"] } as const)
const message = (value: { readonly "left": string; readonly "right": string }) => (value)["left"]
export const main = (_unit: undefined) => _ssrg_console_println(message(swap(duplicate("Imported alias"))))

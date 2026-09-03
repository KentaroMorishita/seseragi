import { reject, type InputError } from "./provider.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export const rejectViaFacade = (input: string) => reject(input)

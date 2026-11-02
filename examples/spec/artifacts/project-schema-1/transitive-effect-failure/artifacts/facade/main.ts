import { reject, type InputError } from "./provider.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source85 = { file: "project/transitive-effect-failure/src/facade.ssrg", line: 4, column: 3, uri: "seseragi://fixture/transitive-effect-failure::facade", start: 85, end: 97 };
$ssrg$assertUnicodeVersion("17.0.0")

export const rejectViaFacade = (input: string) => $ssrg$atSource($ssrg$source85, () => reject(input), true)

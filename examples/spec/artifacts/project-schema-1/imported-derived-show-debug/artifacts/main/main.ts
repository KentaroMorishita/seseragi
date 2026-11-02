import { type Code, type Remote, __ssrg$instance$Debug$1, __ssrg$instance$Debug$3, __ssrg$instance$Show$0, __ssrg$instance$Show$2, type $ssrg$kind$Remote } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source89 = { file: "project/imported-derived-show-debug/src/main.ssrg", line: 4, column: 3, uri: "seseragi://fixture/imported-derived-show-debug::main", start: 89, end: 99 };
const $ssrg$source150 = { file: "project/imported-derived-show-debug/src/main.ssrg", line: 7, column: 3, uri: "seseragi://fixture/imported-derived-show-debug::main", start: 150, end: 161 };
$ssrg$assertUnicodeVersion("17.0.0")

export const render = (value: Remote<Code>) => $ssrg$atSource($ssrg$source89, () => ((__ssrg$instance$Show$2<Code>(__ssrg$instance$Show$0)["show"](value)) as string), false)
export const inspect = (value: Remote<Code>) => $ssrg$atSource($ssrg$source150, () => ((__ssrg$instance$Debug$3<Code>(__ssrg$instance$Debug$1)["debug"](value)) as string), false)

import { type Code, type Remote, __ssrg$instance$Debug$1, __ssrg$instance$Debug$3, __ssrg$instance$Show$0, __ssrg$instance$Show$2 } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export const render = (value: Remote<Code>) => ((__ssrg$instance$Show$2<Code>(__ssrg$instance$Show$0)["show"](value)) as string)
export const inspect = (value: Remote<Code>) => ((__ssrg$instance$Debug$3<Code>(__ssrg$instance$Debug$1)["debug"](value)) as string)

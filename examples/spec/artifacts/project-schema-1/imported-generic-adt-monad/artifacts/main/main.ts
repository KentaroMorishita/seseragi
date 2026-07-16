import { type Box, Boxed, bind, __ssrg$instance$Applicative$1, __ssrg$instance$Functor$0, __ssrg$instance$Monad$2 } from "./domain.js"
import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const incrementBox = (value: bigint) => Boxed(_ssrg_int64_add(value, 1n))
const render = (value: Box<bigint>) => (($ssrg_match: Box<bigint>): string => $ssrg_match.tag === "Boxed" && $ssrg_match.value === 42n ? "Imported Box Monad: 42" : "Imported Box Monad: another value")(value)
const addBox = (left: Box<bigint>) => (right: Box<bigint>) => __ssrg$instance$Monad$2(__ssrg$instance$Applicative$1(__ssrg$instance$Functor$0))["flatMap"]((first: bigint) => __ssrg$instance$Monad$2(__ssrg$instance$Applicative$1(__ssrg$instance$Functor$0))["flatMap"]((second: bigint) => Boxed(_ssrg_int64_add(first, second)))(right))(left)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(render(bind(incrementBox)(Boxed(41n))(__ssrg$instance$Monad$2(__ssrg$instance$Applicative$1(__ssrg$instance$Functor$0))))), () => _ssrg_console_println(render(addBox(Boxed(20n))(Boxed(22n)))))

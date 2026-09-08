import { create, read, unpack, update, box, unbox, nested, collect, type Box, type Secret } from "./facade.js"
import { __ssrg$instance$Eq$0, __ssrg$instance$Show$1 } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { boolShow as _ssrg_show_boolShow, intShow as _ssrg_show_intShow, arrayShow as _ssrg_show_arrayShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => (() => { const secret: Secret = create(7); return _ssrg_effect_flatMap(_ssrg_console_println(read(secret)), () => _ssrg_effect_flatMap(_ssrg_console_println(unpack(secret)), () => _ssrg_effect_flatMap(_ssrg_console_println(((__ssrg$instance$Show$1["show"](secret)) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_boolShow["show"](__ssrg$instance$Eq$0["eq"](secret)(create(7)))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(unbox(box(42))), () => _ssrg_effect_flatMap(_ssrg_console_println(nested(box(secret))), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_arrayShow<number>(_ssrg_show_intShow)["show"](collect([secret]))) as string)), () => _ssrg_console_println(read(update(secret)))))))))); })()

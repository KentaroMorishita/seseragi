import {
  error,
  errorLine,
  print,
  println,
  printValue,
} from "../src/console-service"
import { boolShow } from "../src/show"

print("text")
println("text")
error("text")
errorLine("text")
printValue(true, boolShow)

// @ts-expect-error Console takes String, not arbitrary JS values.
print(true)
// @ts-expect-error Console takes String, not arbitrary JS values.
println(42)
// @ts-expect-error stderr has the same String contract.
error({ toString: () => "text" })
// @ts-expect-error Char and String share JS representation; numbers do not.
errorLine(1.5)

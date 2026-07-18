import * as browserConsole from "../../../../runtime/ts/src/browser/console"
import * as browserStdin from "../../../../runtime/ts/src/browser/stdin"
import * as effect from "../../../../runtime/ts/src/effect"
import * as int64 from "../../../../runtime/ts/src/int64"
import * as iterator from "../../../../runtime/ts/src/iterator"
import * as list from "../../../../runtime/ts/src/list"
import * as array from "../../../../runtime/ts/src/array"
import * as range from "../../../../runtime/ts/src/range"
import * as service from "../../../../runtime/ts/src/service"
import * as show from "../../../../runtime/ts/src/show"
import * as sum from "../../../../runtime/ts/src/sum"
import * as html from "../../../../runtime/ts/src/html"

export const runtimeModules: Readonly<Record<string, unknown>> = {
  "@seseragi/runtime/array": array,
  "@seseragi/runtime/effect": effect,
  "@seseragi/runtime/int64": int64,
  "@seseragi/runtime/iterator": iterator,
  "@seseragi/runtime/list": list,
  "@seseragi/runtime/range": range,
  "@seseragi/runtime/service": service,
  "@seseragi/runtime/show": show,
  "@seseragi/runtime/sum": sum,
  "@seseragi/runtime/html": html,
  "@seseragi/runtime/console": browserConsole,
  "@seseragi/runtime/stdin": browserStdin,
}

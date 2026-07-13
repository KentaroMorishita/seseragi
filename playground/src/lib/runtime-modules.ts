import * as effect from "../../../runtime/ts/src/effect"
import * as int64 from "../../../runtime/ts/src/int64"
import * as service from "../../../runtime/ts/src/service"
import * as show from "../../../runtime/ts/src/show"
import * as sum from "../../../runtime/ts/src/sum"
import * as console from "../../../runtime/ts/src/browser/console"
import * as stdin from "../../../runtime/ts/src/browser/stdin"

export const runtimeModules: Readonly<Record<string, unknown>> = {
  "@seseragi/runtime/effect": effect,
  "@seseragi/runtime/int64": int64,
  "@seseragi/runtime/service": service,
  "@seseragi/runtime/show": show,
  "@seseragi/runtime/sum": sum,
  "@seseragi/runtime/console": console,
  "@seseragi/runtime/stdin": stdin,
}

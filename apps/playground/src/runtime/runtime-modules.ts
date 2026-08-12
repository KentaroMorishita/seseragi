import * as browserConsole from "../../../../runtime/ts/src/browser/console"
import * as browserStdin from "../../../../runtime/ts/src/browser/stdin"
import * as effect from "../../../../runtime/ts/src/effect"
import * as int from "../../../../runtime/ts/src/int"
import * as float from "../../../../runtime/ts/src/float"
import * as iterator from "../../../../runtime/ts/src/iterator"
import * as list from "../../../../runtime/ts/src/list"
import * as number from "../../../../runtime/ts/src/number"
import * as array from "../../../../runtime/ts/src/array"
import * as collection from "../../../../runtime/ts/src/collection"
import * as range from "../../../../runtime/ts/src/range"
import * as service from "../../../../runtime/ts/src/service"
import * as show from "../../../../runtime/ts/src/show"
import * as sum from "../../../../runtime/ts/src/sum"
import * as html from "../../../../runtime/ts/src/html"
import * as dom from "../../../../runtime/ts/src/dom"
import * as signal from "../../../../runtime/ts/src/signal"
import * as string from "../../../../runtime/ts/src/string"
import * as clock from "../../../../runtime/ts/src/clock"
import * as httpClient from "../../../../runtime/ts/src/http-client"
import * as browserClockProvider from "../../../../runtime/ts/src/browser/provider-clock"
import * as browserHttpClientProvider from "../../../../runtime/ts/src/browser/provider-http-client"

export const runtimeModules: Readonly<Record<string, unknown>> = {
  "@seseragi/runtime/array": array,
  "@seseragi/runtime/collection": collection,
  "@seseragi/runtime/effect": effect,
  "@seseragi/runtime/float": float,
  "@seseragi/runtime/int": int,
  "@seseragi/runtime/iterator": iterator,
  "@seseragi/runtime/list": list,
  "@seseragi/runtime/number": number,
  "@seseragi/runtime/range": range,
  "@seseragi/runtime/service": service,
  "@seseragi/runtime/show": show,
  "@seseragi/runtime/sum": sum,
  "@seseragi/runtime/html": html,
  "@seseragi/runtime/dom": dom,
  "@seseragi/runtime/signal": signal,
  "@seseragi/runtime/console": browserConsole,
  "@seseragi/runtime/stdin": browserStdin,
  "@seseragi/runtime/string": string,
  "@seseragi/runtime/clock": clock,
  "@seseragi/runtime/http-client": httpClient,
  "seseragi/runtime-browser/clock": browserClockProvider,
  "seseragi/runtime-browser/http-client": browserHttpClientProvider,
}

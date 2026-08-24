import { createProviderClock } from "../provider-clock"
import { createProviderHttpClient } from "../provider-http-client"
import { createProviderNavigation } from "../provider-navigation"
import { ProviderPackageLoader } from "../provider-package"
import { createProviderStorage } from "../provider-storage"
import { createProviderWebSocketClient } from "../provider-websocket"

export type BrowserProviderSelection = Readonly<{
  provider: string
  service: string
  target: "browser"
  entryModule: string
  entryExport: string
}>

export type BrowserProviderServices = Readonly<{
  clock?: ReturnType<typeof createProviderClock>
  httpClient?: ReturnType<typeof createProviderHttpClient>
  webSocketClient?: ReturnType<typeof createProviderWebSocketClient>
  navigation?: ReturnType<typeof createProviderNavigation>
  storage?: ReturnType<typeof createProviderStorage>
}>

export type BrowserProviderRuntime = Readonly<{
  services: BrowserProviderServices
  shutdown: () => Promise<void>
}>

export async function startBrowserProviders(
  selections: readonly BrowserProviderSelection[],
  importModule: (specifier: string) => Promise<unknown>
): Promise<BrowserProviderRuntime> {
  const loader = new ProviderPackageLoader(
    "browser",
    selections.map((selection) => ({
      provider: selection.provider,
      service: selection.service,
      target: selection.target,
      module: selection.entryModule,
      exportName: selection.entryExport,
      loadMode: "eager" as const,
      importModule: () => importModule(selection.entryModule),
    }))
  )
  try {
    await loader.start()
    const services: {
      clock?: ReturnType<typeof createProviderClock>
      httpClient?: ReturnType<typeof createProviderHttpClient>
      webSocketClient?: ReturnType<typeof createProviderWebSocketClient>
      navigation?: ReturnType<typeof createProviderNavigation>
      storage?: ReturnType<typeof createProviderStorage>
    } = {}
    for (const selection of selections) {
      const loaded = await loader.load(selection.provider)
      switch (selection.service) {
        case "std/clock::Clock":
          services.clock = createProviderClock(loaded)
          break
        case "std/http::HttpClient":
          services.httpClient = createProviderHttpClient(loaded)
          break
        case "std/websocket::WebSocketClient":
          services.webSocketClient = createProviderWebSocketClient(loaded)
          break
        case "std/web/navigation::Navigation":
          services.navigation = createProviderNavigation(loaded)
          break
        case "std/web/storage::Storage":
          services.storage = createProviderStorage(loaded)
          break
      }
    }
    return Object.freeze({
      services: Object.freeze(services),
      shutdown: () => loader.shutdown(),
    })
  } catch (error) {
    await loader.shutdown()
    throw error
  }
}

import {
  createWebSocketClientProvider,
  type WebSocketHostConstructor,
} from "../websocket-host-provider"

const Host = globalThis.WebSocket

export const provider = createWebSocketClientProvider(
  "seseragi/runtime-browser#websocket-client",
  "browser",
  Host as unknown as WebSocketHostConstructor
)

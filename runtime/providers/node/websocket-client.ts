import {
  createWebSocketClientProvider,
  type WebSocketHostConstructor,
} from "@seseragi/runtime/websocket-host-provider"

export const provider = createWebSocketClientProvider(
  "seseragi/runtime-node#websocket-client",
  "node-process",
  globalThis.WebSocket as unknown as WebSocketHostConstructor
)

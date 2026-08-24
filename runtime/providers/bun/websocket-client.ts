import {
  createWebSocketClientProvider,
  type WebSocketHostConstructor,
} from "@seseragi/runtime/websocket-host-provider"

export const provider = createWebSocketClientProvider(
  "seseragi/runtime-bun#websocket-client",
  "bun-process",
  globalThis.WebSocket as unknown as WebSocketHostConstructor
)

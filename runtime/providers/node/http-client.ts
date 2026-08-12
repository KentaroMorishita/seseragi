import { createFetchHttpClientProvider } from "../http-client"

export const provider = createFetchHttpClientProvider(
  "seseragi/runtime-node#http-client",
  "node-process"
)

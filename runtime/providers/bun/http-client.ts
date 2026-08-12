import { createFetchHttpClientProvider } from "../http-client"

export const provider = createFetchHttpClientProvider(
  "seseragi/runtime-bun#http-client",
  "bun-process"
)

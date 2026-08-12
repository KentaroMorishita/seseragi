import { createFileSystemProvider } from "../filesystem"

export const provider = createFileSystemProvider(
  "seseragi/runtime-bun#filesystem",
  "bun-process"
)

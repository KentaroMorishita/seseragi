import { serviceSuccess } from "../service"
import { createByteStdin, type Stdin } from "../stdin-service"
import { Just, Nothing } from "../sum"

export * from "../stdin-service"

export function createTextStdin(input: string): Stdin {
  const bytes = new TextEncoder().encode(input)
  let cursor = 0
  return createByteStdin({
    read(size) {
      if (cursor >= bytes.length) return serviceSuccess(Nothing)
      const end = Math.min(cursor + size, bytes.length)
      const value = bytes.slice(cursor, end)
      cursor = end
      return serviceSuccess(Just(value))
    },
  })
}

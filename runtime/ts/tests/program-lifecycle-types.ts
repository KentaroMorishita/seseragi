import { createProgramLifetime } from "../src/browser/program-lifecycle.js"

function expectType<T>(_value: T) {}

// A consumer must be able to await its owned Effect's actual disposal result,
// while existing pure consumers retain their synchronous return type.
export async function closeEffect(release: () => Promise<void>) {
  const lifetime = createProgramLifetime()
  const owner = await lifetime.load(
    async () => ({ exports: {}, dispose: release, ownership: () => ({}) }),
    () => new Error("closed")
  )
  const completion: Promise<void> = owner.dispose()
  await completion
  await lifetime.close()
}

export async function closePure(release: () => void) {
  const lifetime = createProgramLifetime()
  const owner = await lifetime.load(
    async () => ({ exports: {}, dispose: release, ownership: () => ({}) }),
    () => new Error("closed")
  )
  expectType<void>(owner.dispose())
  await lifetime.close()
}

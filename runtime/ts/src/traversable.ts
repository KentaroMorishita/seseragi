export type RuntimeDictionary = Readonly<
  Record<string, (...args: any[]) => any>
>

type Tree<B> =
  | Readonly<{ tag: "leaf"; value: B }>
  | Readonly<{ tag: "branch"; left: Tree<B>; right: Tree<B> }>

type ApplicativeDictionary = Readonly<{
  map: <A, B>(f: (value: A) => B) => (wrapped: unknown) => unknown
  pure: <A>(value: A) => unknown
  apply: (wrappedFunction: unknown) => (wrappedValue: unknown) => unknown
}>

/**
 * Traverse source-ordered values using only the selected target Applicative.
 * The result is erased because TypeScript cannot express the source HKT G<Shape>;
 * its concrete type is checked by Seseragi and annotated in generated callers.
 */
export function traverseValues<A, B, Shape>(
  values: ReadonlyArray<A>,
  f: (value: A) => unknown,
  applicativeEvidence: RuntimeDictionary,
  finish: (values: ReadonlyArray<B>) => Shape
): any {
  const applicative = applicativeEvidence as ApplicativeDictionary
  // A balanced, persistent tree keeps construction linear, branches independent,
  // and deferred Applicative execution from growing a linear JavaScript stack.
  const combine = (left: unknown, right: unknown): unknown =>
    applicative.apply(
      applicative.map(
        (left: Tree<B>) =>
          (right: Tree<B>): Tree<B> => ({
            tag: "branch",
            left,
            right,
          })
      )(left)
    )(right)
  const levels: Array<{ wrapped: unknown } | undefined> = []

  for (const value of values) {
    let wrapped = applicative.map(
      (value: B): Tree<B> => ({ tag: "leaf", value })
    )(f(value))
    let level = 0
    while (levels[level] !== undefined) {
      wrapped = combine(levels[level]!.wrapped, wrapped)
      levels[level] = undefined
      level += 1
    }
    levels[level] = { wrapped }
  }

  let accumulated: { wrapped: unknown } | undefined
  for (let level = levels.length - 1; level >= 0; level -= 1) {
    const current = levels[level]
    if (current === undefined) continue
    accumulated =
      accumulated === undefined
        ? current
        : { wrapped: combine(accumulated.wrapped, current.wrapped) }
  }
  if (accumulated === undefined) return applicative.pure(finish([]))

  return applicative.map((tree: Tree<B>) => {
    const result: B[] = []
    const pending = [tree]
    while (pending.length > 0) {
      const current = pending.pop()!
      if (current.tag === "leaf") result.push(current.value)
      else pending.push(current.right, current.left)
    }
    return finish(result)
  })(accumulated.wrapped)
}

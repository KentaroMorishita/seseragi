export type Reducible<Collection, Element> = Readonly<{
  reduce: <Accumulator>(
    initial: Accumulator
  ) => (
    step: (accumulator: Accumulator) => (value: Element) => Accumulator
  ) => (values: Collection) => Accumulator
}>

type RuntimeDictionary = Readonly<Record<string, (...arguments_: any[]) => any>>

/** Join a reducible collection without depending on its concrete representation. */
export function join<Collection>(
  dictionary: RuntimeDictionary,
  separator: string,
  values: Collection
): string {
  const reducible = dictionary as Reducible<Collection, string>
  const [, result] = reducible.reduce<readonly [boolean, string]>([true, ""])(
    ([first, output]) =>
      (value) =>
        [false, first ? value : `${output}${separator}${value}`] as const
  )(values)
  return result
}

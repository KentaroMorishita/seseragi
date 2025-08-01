// Seseragi runtime helpers

// ========================================
// 型定義
// ========================================

export type Unit = { tag: "Unit"; value: undefined }
export type Maybe<T> = { tag: "Just"; value: T } | { tag: "Nothing" }
export type Either<L, R> =
  | { tag: "Left"; value: L }
  | { tag: "Right"; value: R }
export type List<T> = { tag: "Empty" } | { tag: "Cons"; head: T; tail: List<T> }
export type Task<T> = { tag: "Task"; computation: () => Promise<T> }
export type Signal<T> = {
  readonly isSignal: true
  readonly getValue: () => T
  readonly setValue: (value: T) => void
  readonly subscribe: (observer: (value: T) => void) => string
  readonly unsubscribe: (key: string) => void
  readonly detach: () => void
  readonly detachHandlers: (() => void)[]
}

// ========================================
// 基本ユーティリティ関数
// ========================================

export function pipe<T, U>(value: T, fn: (arg: T) => U): U {
  return fn(value)
}

export function reversePipe<T, U>(fn: (arg: T) => U, value: T): U {
  return fn(value)
}

// ========================================
// 汎用モナド関数
// ========================================

export function map<T, U>(
  fn: (value: T) => U,
  container: Maybe<T> | Either<unknown, T>
): Maybe<U> | Either<unknown, U> {
  if ("tag" in container) {
    if (container.tag === "Just") return Just(fn(container.value))
    if (container.tag === "Right") return Right(fn(container.value))
    if (container.tag === "Nothing") return Nothing
    if (container.tag === "Left") return container
  }
  return Nothing
}

export function applyWrapped<T, U>(
  wrapped: Maybe<(value: T) => U> | Either<unknown, (value: T) => U>,
  container: Maybe<T> | Either<unknown, T>
): Maybe<U> | Either<unknown, U> {
  // Maybe types
  if (wrapped.tag === "Nothing" || container.tag === "Nothing") return Nothing
  if (wrapped.tag === "Just" && container.tag === "Just")
    return Just(wrapped.value(container.value))
  // Either types
  if (wrapped.tag === "Left") return wrapped
  if (container.tag === "Left") return container
  if (wrapped.tag === "Right" && container.tag === "Right")
    return Right(wrapped.value(container.value))
  return Nothing
}

export function bind<T, U>(
  container: Maybe<T> | Either<unknown, T>,
  fn: (value: T) => Maybe<U> | Either<unknown, U>
): Maybe<U> | Either<unknown, U> {
  if (container.tag === "Just") return fn(container.value)
  if (container.tag === "Right") return fn(container.value)
  if (container.tag === "Nothing") return Nothing
  if (container.tag === "Left") return container
  return Nothing
}

export function foldMonoid<T>(
  arr: T[],
  empty: T,
  combine: (a: T, b: T) => T
): T {
  return arr.reduce(combine, empty)
}

// ========================================
// Unit型
// ========================================

export const Unit: Unit = { tag: "Unit", value: undefined }

// ========================================
// Maybe型
// ========================================

export function Just<T>(value: T): Maybe<T> {
  return { tag: "Just", value }
}

export const Nothing: Maybe<never> = { tag: "Nothing" }

export function mapMaybe<T, U>(fa: Maybe<T>, f: (a: T) => U): Maybe<U> {
  return fa.tag === "Just" ? Just(f(fa.value)) : Nothing
}

export function applyMaybe<T, U>(
  ff: Maybe<(a: T) => U>,
  fa: Maybe<T>
): Maybe<U> {
  return ff.tag === "Just" && fa.tag === "Just"
    ? Just(ff.value(fa.value))
    : Nothing
}

export function bindMaybe<T, U>(
  ma: Maybe<T>,
  f: (value: T) => Maybe<U>
): Maybe<U> {
  return ma.tag === "Just" ? f(ma.value) : Nothing
}

export function fromMaybe<T>(defaultValue: T, maybe: Maybe<T>): T {
  return maybe.tag === "Just" ? maybe.value : defaultValue
}

// ========================================
// Either型
// ========================================

export function Left<L>(value: L): Either<L, never> {
  return { tag: "Left", value }
}

export function Right<R>(value: R): Either<never, R> {
  return { tag: "Right", value }
}

export function mapEither<L, R, U>(
  ea: Either<L, R>,
  f: (value: R) => U
): Either<L, U> {
  return ea.tag === "Right" ? Right(f(ea.value)) : ea
}

export function applyEither<L, R, U>(
  ef: Either<L, (value: R) => U>,
  ea: Either<L, R>
): Either<L, U> {
  return ef.tag === "Right" && ea.tag === "Right"
    ? Right(ef.value(ea.value))
    : ef.tag === "Left"
      ? ef
      : (ea as Either<L, U>)
}

export function bindEither<L, R, U>(
  ea: Either<L, R>,
  f: (value: R) => Either<L, U>
): Either<L, U> {
  return ea.tag === "Right" ? f(ea.value) : ea
}

export function fromRight<L, R>(defaultValue: R, either: Either<L, R>): R {
  return either.tag === "Right" ? either.value : defaultValue
}

export function fromLeft<L, R>(defaultValue: L, either: Either<L, R>): L {
  return either.tag === "Left" ? either.value : defaultValue
}

// ========================================
// List型
// ========================================

export const Empty: List<never> = { tag: "Empty" }

export function Cons<T>(head: T, tail: List<T>): List<T> {
  return { tag: "Cons", head, tail }
}

export function headList<T>(list: List<T>): Maybe<T> {
  return list.tag === "Cons"
    ? { tag: "Just", value: list.head }
    : { tag: "Nothing" }
}

export function tailList<T>(list: List<T>): List<T> {
  return list.tag === "Cons" ? list.tail : Empty
}

export function mapList<T, U>(fa: List<T>, f: (a: T) => U): List<U> {
  if (fa.tag === "Empty") return { tag: "Empty" }
  return { tag: "Cons", head: f(fa.head), tail: mapList(fa.tail, f) }
}

export function applyList<T, U>(ff: List<(a: T) => U>, fa: List<T>): List<U> {
  if (ff.tag === "Empty") return { tag: "Empty" }
  const mappedValues = mapList(fa, ff.head)
  const restApplied = applyList(ff.tail, fa)
  return concatList(mappedValues, restApplied)
}

export function concatList<T>(list1: List<T>, list2: List<T>): List<T> {
  if (list1.tag === "Empty") return list2
  return { tag: "Cons", head: list1.head, tail: concatList(list1.tail, list2) }
}

export function bindList<T, U>(ma: List<T>, f: (value: T) => List<U>): List<U> {
  if (ma.tag === "Empty") return { tag: "Empty" }
  const headResult = f(ma.head)
  const tailResult = bindList(ma.tail, f)
  return concatList(headResult, tailResult)
}

// ========================================
// Array型
// ========================================

export function mapArray<T, U>(fa: T[], f: (a: T) => U): U[] {
  return fa.map(f)
}

export function applyArray<T, U>(ff: ((a: T) => U)[], fa: T[]): U[] {
  const result: U[] = []
  for (const func of ff) {
    for (const value of fa) {
      result.push(func(value))
    }
  }
  return result
}

export function bindArray<T, U>(ma: T[], f: (value: T) => U[]): U[] {
  const result: U[] = []
  for (const value of ma) {
    result.push(...f(value))
  }
  return result
}

export function arrayToList<T>(arr: T[]): List<T> {
  let result: List<T> = Empty
  for (let i = arr.length - 1; i >= 0; i--) {
    result = Cons(arr[i], result)
  }
  return result
}

export function listToArray<T>(list: List<T>): T[] {
  const result: T[] = []
  let current = list
  while (current.tag === "Cons") {
    result.push(current.head)
    current = current.tail
  }
  return result
}

// ========================================
// Task型
// ========================================

export function Task<T>(computation: () => Promise<T>): Task<T> {
  return { tag: "Task", computation }
}

export function resolve<T>(value: T): () => Promise<T> {
  return () => Promise.resolve(value)
}

export function ssrgRun<T>(task: Task<T>): Promise<T> {
  return task.computation()
}

export function ssrgTryRun<T>(task: Task<T>): Promise<Either<string, T>> {
  return (async () => {
    try {
      const result = await task.computation()
      return Right(result)
    } catch (error) {
      return Left(error instanceof Error ? error.message : String(error))
    }
  })()
}

export function mapTask<A, B>(f: (a: A) => B, fa: Task<A>): Task<B> {
  return Task(async () => {
    const a = await fa.computation()
    return f(a)
  })
}

export function applyTask<A, B>(ff: Task<(a: A) => B>, fa: Task<A>): Task<B> {
  return Task(async () => {
    const [f, a] = await Promise.all([ff.computation(), fa.computation()])
    return f(a)
  })
}

export function bindTask<A, B>(ma: Task<A>, f: (a: A) => Task<B>): Task<B> {
  return Task(async () => {
    const a = await ma.computation()
    const mb = f(a)
    return mb.computation()
  })
}

// ========================================
// Signal型
// ========================================

export function createSignal<T>(initialValue: T): Signal<T> {
  let currentValue = initialValue
  const observers: Map<string, (value: T) => void> = new Map()
  let keyCounter = 0
  const detachHandlers: (() => void)[] = []

  const signal: Signal<T> = {
    isSignal: true,
    getValue: () => currentValue,
    setValue: (value: T) => {
      currentValue = value
      for (const observer of observers.values()) {
        observer(currentValue)
      }
    },
    subscribe: (observer: (value: T) => void) => {
      const key = `observer_${keyCounter++}`
      observers.set(key, observer)
      return key
    },
    unsubscribe: (key: string) => {
      observers.delete(key)
    },
    detach: () => {
      for (const detachHandler of detachHandlers) {
        detachHandler()
      }
      detachHandlers.length = 0
    },
    detachHandlers,
  }

  return signal
}

export function setSignal<T>(signal: Signal<T>, value: T): void {
  signal.setValue(value)
}

export function subscribeSignal<T>(
  signal: Signal<T>,
  observer: (value: T) => void
): string {
  return signal.subscribe(observer)
}

export function unsubscribeSignal<T>(signal: Signal<T>, key: string): void {
  signal.unsubscribe(key)
}

export function detachSignal<T>(signal: Signal<T>): void {
  signal.detach()
}

export function mapSignal<T, U>(
  sourceSignal: Signal<T>,
  f: (value: T) => U
): Signal<U> {
  const resultSignal = createSignal(f(sourceSignal.getValue()))
  const subscriptionKey = sourceSignal.subscribe((newValue) => {
    resultSignal.setValue(f(newValue))
  })
  resultSignal.detachHandlers.push(() =>
    sourceSignal.unsubscribe(subscriptionKey)
  )
  return resultSignal
}

export function applySignal<T, U>(
  fnSignal: Signal<(value: T) => U>,
  valueSignal: Signal<T>
): Signal<U> {
  const resultSignal = createSignal(fnSignal.getValue()(valueSignal.getValue()))
  const fnSubscriptionKey = fnSignal.subscribe((newFn) => {
    if (typeof newFn === "function") {
      resultSignal.setValue(newFn(valueSignal.getValue()))
    } else {
      console.error(
        "applySignal: newFn is not a function:",
        newFn,
        typeof newFn
      )
    }
  })
  const valueSubscriptionKey = valueSignal.subscribe((newValue) => {
    const currentFn = fnSignal.getValue()
    if (typeof currentFn === "function") {
      resultSignal.setValue(currentFn(newValue))
    } else {
      console.error(
        "applySignal: currentFn is not a function:",
        currentFn,
        typeof currentFn
      )
    }
  })
  resultSignal.detachHandlers.push(() =>
    fnSignal.unsubscribe(fnSubscriptionKey)
  )
  resultSignal.detachHandlers.push(() =>
    valueSignal.unsubscribe(valueSubscriptionKey)
  )
  return resultSignal
}

export function bindSignal<T, U>(
  sourceSignal: Signal<T>,
  f: (value: T) => Signal<U>
): Signal<U> {
  let currentInnerSignal = f(sourceSignal.getValue())
  const resultSignal = createSignal(currentInnerSignal.getValue())
  const forwardInnerValue = (newValue: U) => resultSignal.setValue(newValue)
  const switchToNewInnerSignal = (newSourceValue: T) => {
    const newInnerSignal = f(newSourceValue)
    currentInnerSignal = newInnerSignal
    resultSignal.setValue(newInnerSignal.getValue())
    const newInnerSubscriptionKey = newInnerSignal.subscribe(forwardInnerValue)
    resultSignal.detachHandlers.push(() =>
      newInnerSignal.unsubscribe(newInnerSubscriptionKey)
    )
  }
  const sourceSubscriptionKey = sourceSignal.subscribe(switchToNewInnerSignal)
  resultSignal.detachHandlers.push(() =>
    sourceSignal.unsubscribe(sourceSubscriptionKey)
  )
  const initialInnerSubscriptionKey =
    currentInnerSignal.subscribe(forwardInnerValue)
  resultSignal.detachHandlers.push(() =>
    currentInnerSignal.unsubscribe(initialInnerSubscriptionKey)
  )
  return resultSignal
}

// グローバルなサブスクリプション管理
const __signalSubscriptions: Map<string, { signal: Signal<any>; key: string }> =
  new Map()

export function ssrgSignalSubscribe<T>(
  signal: Signal<T>,
  observer: (value: T) => void
): string {
  const subscriptionKey = signal.subscribe(observer)
  const globalKey = `${Date.now()}_${Math.random()}`
  __signalSubscriptions.set(globalKey, { signal, key: subscriptionKey })
  return globalKey
}

export function ssrgSignalUnsubscribe(globalKey: string): void {
  const subscription = __signalSubscriptions.get(globalKey)
  if (subscription) {
    subscription.signal.unsubscribe(subscription.key)
    __signalSubscriptions.delete(globalKey)
  }
}

export function ssrgSignalDetach<T>(signal: Signal<T>): void {
  signal.detach()
}

// ========================================
// 組み込み関数
// ========================================

export function ssrgPrint(value: unknown): void {
  // Seseragi型の場合は美しく整形
  if (
    value &&
    typeof value === "object" &&
    ((value as any).tag === "Unit" ||
      (value as any).tag === "Just" ||
      (value as any).tag === "Nothing" ||
      (value as any).tag === "Left" ||
      (value as any).tag === "Right" ||
      (value as any).tag === "Cons" ||
      (value as any).tag === "Empty")
  ) {
    console.log(ssrgToString(value))
  }
  // 通常のオブジェクトはそのまま
  else {
    console.log(value)
  }
}

export function ssrgPutStrLn(value: string): void {
  console.log(value)
}

export function ssrgToString(value: unknown): string {
  // Unit型の美しい表示
  if (value && typeof value === "object" && (value as any).tag === "Unit") {
    return "()"
  }

  // Maybe型の美しい表示
  if (value && typeof value === "object" && (value as any).tag === "Just") {
    return `Just(${ssrgToString((value as any).value)})`
  }
  if (value && typeof value === "object" && (value as any).tag === "Nothing") {
    return "Nothing"
  }

  // Either型の美しい表示
  if (value && typeof value === "object" && (value as any).tag === "Left") {
    return `Left(${ssrgToString((value as any).value)})`
  }
  if (value && typeof value === "object" && (value as any).tag === "Right") {
    return `Right(${ssrgToString((value as any).value)})`
  }

  // List型の美しい表示
  if (value && typeof value === "object" && (value as any).tag === "Empty") {
    return "`[]"
  }
  if (value && typeof value === "object" && (value as any).tag === "Cons") {
    const items: string[] = []
    let current = value as any
    while (current.tag === "Cons") {
      items.push(ssrgToString(current.head))
      current = current.tail
    }
    return `\`[${items.join(", ")}]`
  }

  // Tuple型の美しい表示
  if (value && typeof value === "object" && (value as any).tag === "Tuple") {
    return `(${(value as any).elements.map(ssrgToString).join(", ")})`
  }

  // 配列の表示
  if (Array.isArray(value)) {
    return `[${value.map(ssrgToString).join(", ")}]`
  }

  // プリミティブ型
  if (typeof value === "string") {
    return `"${value}"`
  }
  if (typeof value === "number") {
    return String(value)
  }
  if (typeof value === "boolean") {
    return value ? "True" : "False"
  }

  // 普通のオブジェクト（構造体など）
  if (typeof value === "object" && value !== null) {
    const pairs: string[] = []
    for (const key in value) {
      if (Object.hasOwn(value as any, key)) {
        pairs.push(`${key}: ${ssrgToString((value as any)[key])}`)
      }
    }

    // 構造体名を取得（constructor.nameを使用）
    const structName =
      (value as any).constructor && (value as any).constructor.name !== "Object"
        ? (value as any).constructor.name
        : ""

    // 複数フィールドがある場合はインデント表示
    if (pairs.length > 2) {
      return `${structName} {\n  ${pairs.join(",\n  ")}\n}`
    } else {
      return `${structName} { ${pairs.join(", ")} }`
    }
  }

  return String(value)
}

export function ssrgToInt(value: unknown): number {
  if (typeof value === "number") {
    return Math.trunc(value)
  }
  if (typeof value === "string") {
    const n = parseInt(value, 10)
    if (Number.isNaN(n)) {
      throw new Error(`Cannot convert "${value}" to Int`)
    }
    return n
  }
  throw new Error(`Cannot convert ${typeof value} to Int`)
}

export function ssrgToFloat(value: unknown): number {
  if (typeof value === "number") {
    return value
  }
  if (typeof value === "string") {
    const n = parseFloat(value)
    if (Number.isNaN(n)) {
      throw new Error(`Cannot convert "${value}" to Float`)
    }
    return n
  }
  throw new Error(`Cannot convert ${typeof value} to Float`)
}

export function ssrgShow(value: unknown): void {
  console.log(ssrgToString(value))
}

// ========================================
// 型システム関数
// ========================================

// 型レジストリ（コンパイル時型情報の実行時保持）
export const __typeRegistry: Record<string, any> = {}

// 変数型情報テーブル（完全型情報保持）
export const __variableTypes: Record<string, string> = {}
export const __variableAliases: Record<string, string[]> = {}

export function ssrgTypeOf(value: unknown, variableName?: string): string {
  if (value === null) return "null"
  if (value === undefined) return "undefined"

  // 1. 変数名がある場合は型テーブルから取得
  if (variableName && __variableTypes[variableName]) {
    return __variableTypes[variableName]
  }

  // 2. __typename プロパティをチェック（型エイリアス対応）
  if (value && typeof value === "object" && "__typename" in value) {
    return (value as any).__typename
  }

  // 3. プリミティブ型
  if (typeof value === "string") return "String"
  if (typeof value === "number") return "Int"
  if (typeof value === "boolean") return "Bool"

  // 4. 組み込み型の特別処理
  if (value && typeof value === "object") {
    // Unit型
    if ((value as any).tag === "Unit") {
      return "Unit"
    }

    // Maybe型
    if ((value as any).tag === "Just" || (value as any).tag === "Nothing") {
      if ((value as any).tag === "Just") {
        const innerType = ssrgTypeOf((value as any).value)
        return `Maybe<${innerType}>`
      }
      return "Maybe<unknown>"
    }

    // Either型
    if ((value as any).tag === "Left" || (value as any).tag === "Right") {
      const innerType = ssrgTypeOf((value as any).value)
      if ((value as any).tag === "Left") {
        return `Either<${innerType}, unknown>`
      } else {
        return `Either<unknown, ${innerType}>`
      }
    }

    // Tuple型
    if (
      (value as any).tag === "Tuple" &&
      Array.isArray((value as any).elements)
    ) {
      const elemTypes = (value as any).elements.map((elem: any) =>
        ssrgTypeOf(elem)
      )
      return `(${elemTypes.join(", ")})`
    }

    // Array型
    if (Array.isArray(value)) {
      if (value.length > 0) {
        const elemType = ssrgTypeOf(value[0])
        return `Array<${elemType}>`
      }
      return "Array<unknown>"
    }

    // 4. 構造体の場合はコンストラクタ名を返す
    if (
      (value as any).constructor &&
      (value as any).constructor.name !== "Object"
    ) {
      return (value as any).constructor.name
    }

    // 5. 匿名オブジェクトの場合は型構造を返す（構造的型システム）
    const keys = Object.keys(value as any).sort() // キーをソートして順序を統一
    if (keys.length > 0) {
      const fields = keys
        .map((key) => `${key}: ${ssrgTypeOf((value as any)[key])}`)
        .join(", ")
      return `{ ${fields} }`
    }
  }

  return "unknown"
}

export function ssrgTypeOfWithAliases(
  value: unknown,
  variableName?: string
): string {
  // 構造的型を取得（変数テーブルを使わずに）
  const structuralType = ssrgTypeOf(value) // 変数名なしで呼ぶ

  // 変数エイリアステーブルから取得（優先）
  let matchingAliases: string[] = []
  if (variableName && __variableAliases[variableName]) {
    matchingAliases = __variableAliases[variableName]
  } else {
    // フォールバック: 型レジストリから該当するエイリアスを検索
    if (value && typeof value === "object") {
      const structuralTypeForMatch = getStructuralTypeString(value)
      for (const [typeName, typeInfo] of Object.entries(__typeRegistry)) {
        if (typeMatches(structuralTypeForMatch, typeInfo)) {
          matchingAliases.push(typeName)
        }
      }
    }
  }

  // エイリアスがある場合は追加情報として表示
  if (matchingAliases.length > 0) {
    return `${structuralType} (${matchingAliases.join(", ")})`
  }

  return structuralType
}

export function ssrgIsType(
  value: unknown,
  typeString: string,
  variableName?: string
): boolean {
  // 1. 変数型テーブルからの型情報チェック（ワイルドカードがない場合のみ最優先）
  if (
    variableName &&
    __variableTypes[variableName] &&
    !typeString.includes("_")
  ) {
    const variableType = __variableTypes[variableName]

    // struct型の場合は構造体名のみで比較
    if (variableType.includes(" { ")) {
      const structName = variableType.split(" { ")[0]
      if (structName === typeString) return true
    }

    // 完全一致の場合
    if (variableType === typeString) return true

    // type aliasの場合は変数エイリアステーブルをチェック
    if (variableName && __variableAliases[variableName]) {
      const aliases = __variableAliases[variableName]
      if (aliases.includes(typeString)) {
        return true
      }
    }

    // 構造的型の場合は順序を無視した比較
    if (
      variableType.startsWith("{") &&
      variableType.endsWith("}") &&
      typeString.startsWith("{") &&
      typeString.endsWith("}")
    ) {
      return checkStructuralType(value, typeString)
    }

    return false
  }

  // 2. ワイルドカード型の場合は、変数型テーブルの情報とワイルドカードマッチングを組み合わせる
  if (
    variableName &&
    __variableTypes[variableName] &&
    typeString.includes("_")
  ) {
    const variableType = __variableTypes[variableName]
    return wildcardTypeMatches(variableType, typeString)
  }

  // 2. 直接的な型名マッチ
  const actualType = ssrgTypeOf(value)
  if (actualType === typeString) return true

  // 3. 型レジストリを使った同等性チェック
  const registryType = __typeRegistry[typeString]
  if (registryType) {
    return typeMatchesRegistry(value, registryType)
  }

  // 4. 組み込み型の特別処理
  if (value && typeof value === "object") {
    // Unit型チェック
    if (typeString === "Unit") {
      return (value as any).tag === "Unit"
    }

    // Maybe型チェック
    if (typeString.startsWith("Maybe<")) {
      if ((value as any).tag === "Just" || (value as any).tag === "Nothing") {
        if ((value as any).tag === "Nothing") {
          return true // Nothing は任意の Maybe<T> にマッチ
        }
        // Just の場合は内部型をチェック
        const innerTypeMatch = typeString.match(/Maybe<(.+)>/)
        if (innerTypeMatch) {
          const expectedInnerType = innerTypeMatch[1]
          // ワイルドカードの場合は任意の型にマッチ
          if (expectedInnerType === "_") return true
          return ssrgIsType((value as any).value, expectedInnerType)
        }
      }
      return false
    }

    // Either型チェック
    if (typeString.startsWith("Either<")) {
      if ((value as any).tag === "Left" || (value as any).tag === "Right") {
        // Either<A, B> から A と B を正確に抽出
        const content = typeString.slice(7, -1) // "Either<" と ">" を除く
        const commaIndex = content.indexOf(",")
        if (commaIndex !== -1) {
          const leftType = content.slice(0, commaIndex).trim()
          const rightType = content.slice(commaIndex + 1).trim()

          if ((value as any).tag === "Left") {
            // ワイルドカードの場合は任意の型にマッチ
            if (leftType === "_") return true
            return ssrgIsType((value as any).value, leftType)
          } else {
            // ワイルドカードの場合は任意の型にマッチ
            if (rightType === "_") return true
            return ssrgIsType((value as any).value, rightType)
          }
        }
      }
      return false
    }

    // Array型チェック
    if (typeString.startsWith("Array<")) {
      if (Array.isArray(value)) {
        const typeMatch = typeString.match(/Array<(.+)>/)
        if (typeMatch) {
          const elemType = typeMatch[1]
          // ワイルドカードの場合は任意の型にマッチ
          if (elemType === "_") return true
          return value.every((item) => ssrgIsType(item, elemType))
        }
      }
      return false
    }

    // Tuple型チェック
    if (typeString.startsWith("(") && typeString.endsWith(")")) {
      if (
        (value as any).tag === "Tuple" &&
        Array.isArray((value as any).elements)
      ) {
        const tupleContent = typeString.slice(1, -1)
        const expectedTypes = tupleContent.split(",").map((t) => t.trim())
        const actualElements = (value as any).elements
        if (expectedTypes.length !== actualElements.length) return false
        return expectedTypes.every((expectedType, index) => {
          // ワイルドカードの場合は任意の型にマッチ
          if (expectedType === "_") return true
          return ssrgIsType(actualElements[index], expectedType)
        })
      }
      return false
    }
  }

  // 5. 構造的型チェック（レコード型）
  if (typeString.startsWith("{") && typeString.endsWith("}")) {
    return checkStructuralType(value, typeString)
  }

  return false
}

// ========================================
// 内部ヘルパー関数
// ========================================

function getStructuralTypeString(value: any): string {
  if (!value || typeof value !== "object") return "unknown"
  const keys = Object.keys(value).sort() // キーをソートして順序を統一
  if (keys.length === 0) return "{}"
  const fields = keys
    .map((key) => `${key}: ${ssrgTypeOf(value[key])}`)
    .join(", ")
  return `{ ${fields} }`
}

function typeMatches(structuralType: string, typeInfo: any): boolean {
  if (!typeInfo || typeof typeInfo !== "object") return false

  switch (typeInfo.kind) {
    case "record": {
      const expectedFields = Object.keys(typeInfo.fields)
        .sort() // キーをソートして順序を統一
        .map((name) => `${name}: ${getTypeInfoString(typeInfo.fields[name])}`)
        .join(", ")
      return structuralType === `{ ${expectedFields} }`
    }
    case "tuple": {
      const expectedElements = typeInfo.elements
        .map((elem: any) => getTypeInfoString(elem))
        .join(", ")
      return structuralType === `(${expectedElements})`
    }
    default:
      return false
  }
}

function getTypeInfoString(typeInfo: any): string {
  if (!typeInfo || typeof typeInfo !== "object") return "unknown"

  switch (typeInfo.kind) {
    case "primitive":
      return typeInfo.name || "unknown"
    case "array":
      return `Array<${getTypeInfoString(typeInfo.elementType)}>`
    case "maybe":
      return `Maybe<${getTypeInfoString(typeInfo.innerType)}>`
    case "either":
      return `Either<${getTypeInfoString(typeInfo.leftType)}, ${getTypeInfoString(typeInfo.rightType)}>`
    case "tuple":
      return `(${typeInfo.elements.map((elem: any) => getTypeInfoString(elem)).join(", ")})`
    case "record": {
      const fields = Object.keys(typeInfo.fields)
        .map((name) => `${name}: ${getTypeInfoString(typeInfo.fields[name])}`)
        .join(", ")
      return `{ ${fields} }`
    }
    default:
      return "unknown"
  }
}

function wildcardTypeMatches(
  actualType: string,
  expectedType: string
): boolean {
  // ワイルドカード型マッチング：Either<String, Int> と Either<String, _> などをマッチ
  if (expectedType.startsWith("Either<")) {
    if (!actualType.startsWith("Either<")) return false

    const actualContent = actualType.slice(7, -1)
    const expectedContent = expectedType.slice(7, -1)

    const actualCommaIndex = actualContent.indexOf(",")
    const expectedCommaIndex = expectedContent.indexOf(",")

    if (actualCommaIndex === -1 || expectedCommaIndex === -1) return false

    const actualLeft = actualContent.slice(0, actualCommaIndex).trim()
    const actualRight = actualContent.slice(actualCommaIndex + 1).trim()
    const expectedLeft = expectedContent.slice(0, expectedCommaIndex).trim()
    const expectedRight = expectedContent.slice(expectedCommaIndex + 1).trim()

    // ワイルドカードチェック
    const leftMatches = expectedLeft === "_" || actualLeft === expectedLeft
    const rightMatches = expectedRight === "_" || actualRight === expectedRight

    return leftMatches && rightMatches
  }

  // Maybe型のワイルドカードマッチング
  if (expectedType.startsWith("Maybe<")) {
    if (!actualType.startsWith("Maybe<")) return false

    const actualInner = actualType.slice(6, -1)
    const expectedInner = expectedType.slice(6, -1)

    return expectedInner === "_" || actualInner === expectedInner
  }

  // Array型のワイルドカードマッチング
  if (expectedType.startsWith("Array<")) {
    if (!actualType.startsWith("Array<")) return false

    const actualInner = actualType.slice(6, -1)
    const expectedInner = expectedType.slice(6, -1)

    return expectedInner === "_" || actualInner === expectedInner
  }

  // レコード型のワイルドカードマッチング
  if (expectedType.startsWith("{") && expectedType.endsWith("}")) {
    if (!actualType.startsWith("{") || !actualType.endsWith("}")) return false

    // 両方の型を構造的にパース
    const actualContent = actualType.slice(1, -1).trim()
    const expectedContent = expectedType.slice(1, -1).trim()

    const actualFields: Record<string, string> = {}
    const expectedFields: Record<string, string> = {}

    // actualTypeをパース
    actualContent.split(",").forEach((field) => {
      const colonIndex = field.indexOf(":")
      if (colonIndex !== -1) {
        const name = field.slice(0, colonIndex).trim()
        const type = field.slice(colonIndex + 1).trim()
        actualFields[name] = type
      }
    })

    // expectedTypeをパース
    expectedContent.split(",").forEach((field) => {
      const colonIndex = field.indexOf(":")
      if (colonIndex !== -1) {
        const name = field.slice(0, colonIndex).trim()
        const type = field.slice(colonIndex + 1).trim()
        expectedFields[name] = type
      }
    })

    // フィールド数が一致する必要がある
    if (Object.keys(actualFields).length !== Object.keys(expectedFields).length)
      return false

    // 各フィールドをチェック
    for (const [fieldName, expectedFieldType] of Object.entries(
      expectedFields
    )) {
      if (!(fieldName in actualFields)) return false
      if (expectedFieldType === "_") continue // ワイルドカードは任意の型にマッチ
      if (actualFields[fieldName] !== expectedFieldType) return false
    }

    return true
  }

  // Tuple型のワイルドカードマッチング
  if (expectedType.startsWith("(") && expectedType.endsWith(")")) {
    if (!actualType.startsWith("(") || !actualType.endsWith(")")) return false

    const actualContent = actualType.slice(1, -1).trim()
    const expectedContent = expectedType.slice(1, -1).trim()

    const actualElements = actualContent.split(",").map((e) => e.trim())
    const expectedElements = expectedContent.split(",").map((e) => e.trim())

    // 要素数が一致する必要がある
    if (actualElements.length !== expectedElements.length) return false

    // 各要素をチェック
    for (let i = 0; i < actualElements.length; i++) {
      if (expectedElements[i] === "_") continue // ワイルドカードは任意の型にマッチ
      if (actualElements[i] !== expectedElements[i]) return false
    }

    return true
  }

  return false
}

function typeMatchesRegistry(value: any, typeInfo: any): boolean {
  if (!typeInfo || typeof typeInfo !== "object") return false

  switch (typeInfo.kind) {
    case "primitive":
      return ssrgTypeOf(value) === typeInfo.name
    case "record":
      if (!value || typeof value !== "object") return false
      return Object.keys(typeInfo.fields).every((fieldName) => {
        if (!(fieldName in value)) return false
        return typeMatchesRegistry(value[fieldName], typeInfo.fields[fieldName])
      })
    case "tuple": {
      if (!value || typeof value !== "object" || (value as any).tag !== "Tuple")
        return false
      const elements = (value as any).elements
      if (
        !Array.isArray(elements) ||
        elements.length !== typeInfo.elements.length
      )
        return false
      return typeInfo.elements.every((expectedType: any, index: number) =>
        typeMatchesRegistry(elements[index], expectedType)
      )
    }
    case "array":
      if (!Array.isArray(value)) return false
      return value.every((item) =>
        typeMatchesRegistry(item, typeInfo.elementType)
      )
    case "maybe":
      if (!value || typeof value !== "object") return false
      if ((value as any).tag === "Nothing") return true
      if ((value as any).tag === "Just") {
        return typeMatchesRegistry((value as any).value, typeInfo.innerType)
      }
      return false
    case "either":
      if (!value || typeof value !== "object") return false
      if ((value as any).tag === "Left") {
        return typeMatchesRegistry((value as any).value, typeInfo.leftType)
      }
      if ((value as any).tag === "Right") {
        return typeMatchesRegistry((value as any).value, typeInfo.rightType)
      }
      return false
    default:
      return false
  }
}

function checkStructuralType(value: any, typeString: string): boolean {
  if (!value || typeof value !== "object") return false

  // "{ name: String, age: Int }" のような形式をパース
  const content = typeString.slice(1, -1).trim()
  if (!content) return Object.keys(value).length === 0

  const fields = content.split(",").map((f) => f.trim())
  const expectedFields: Record<string, string> = {}

  for (const field of fields) {
    const colonIndex = field.indexOf(":")
    if (colonIndex === -1) continue
    const fieldName = field.slice(0, colonIndex).trim()
    const fieldType = field.slice(colonIndex + 1).trim()
    expectedFields[fieldName] = fieldType
  }

  // 期待されるフィールドがすべて存在し、型が一致するかチェック
  for (const [fieldName, expectedType] of Object.entries(expectedFields)) {
    if (!(fieldName in value)) return false
    // ワイルドカードの場合は任意の型にマッチ
    if (expectedType === "_") continue
    if (!ssrgIsType(value[fieldName], expectedType)) return false
  }

  // 余分なフィールドがないかチェック
  const actualFieldCount = Object.keys(value).filter(
    (key) => key !== "__typename"
  ).length
  const expectedFieldCount = Object.keys(expectedFields).length
  return actualFieldCount === expectedFieldCount
}

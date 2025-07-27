// Generated TypeScript code from Seseragi

// Seseragi runtime helpers

type Unit = { tag: 'Unit', value: undefined };
type Maybe<T> = { tag: 'Just'; value: T } | { tag: 'Nothing' };
type Either<L, R> = { tag: 'Left'; value: L } | { tag: 'Right'; value: R };
type List<T> = { tag: 'Empty' } | { tag: 'Cons'; head: T; tail: List<T> };


function pipe<T, U>(value: T, fn: (arg: T) => U): U { return fn(value); }

function reversePipe<T, U>(fn: (arg: T) => U, value: T): U { return fn(value); }

function map<T, U>(fn: (value: T) => U, container: Maybe<T> | Either<unknown, T>): Maybe<U> | Either<unknown, U> {
  if ('tag' in container) {
    if (container.tag === 'Just') return Just(fn(container.value));
    if (container.tag === 'Right') return Right(fn(container.value));
    if (container.tag === 'Nothing') return Nothing;
    if (container.tag === 'Left') return container;
  }
  return Nothing;
}

function applyWrapped<T, U>(wrapped: Maybe<(value: T) => U> | Either<unknown, (value: T) => U>, container: Maybe<T> | Either<unknown, T>): Maybe<U> | Either<unknown, U> {
  // Maybe types
  if (wrapped.tag === 'Nothing' || container.tag === 'Nothing') return Nothing;
  if (wrapped.tag === 'Just' && container.tag === 'Just') return Just(wrapped.value(container.value));
  // Either types
  if (wrapped.tag === 'Left') return wrapped;
  if (container.tag === 'Left') return container;
  if (wrapped.tag === 'Right' && container.tag === 'Right') return Right(wrapped.value(container.value));
  return Nothing;
}

function bind<T, U>(container: Maybe<T> | Either<unknown, T>, fn: (value: T) => Maybe<U> | Either<unknown, U>): Maybe<U> | Either<unknown, U> {
  if (container.tag === 'Just') return fn(container.value);
  if (container.tag === 'Right') return fn(container.value);
  if (container.tag === 'Nothing') return Nothing;
  if (container.tag === 'Left') return container;
  return Nothing;
}

function foldMonoid<T>(arr: T[], empty: T, combine: (a: T, b: T) => T): T {
  return arr.reduce(combine, empty);
}

// Array monadic functions
function mapArray<T, U>(fa: T[], f: (a: T) => U): U[] {
  return fa.map(f);
}

function applyArray<T, U>(ff: ((a: T) => U)[], fa: T[]): U[] {
  const result: U[] = [];
  for (const func of ff) {
    for (const value of fa) {
      result.push(func(value));
    }
  }
  return result;
}

function bindArray<T, U>(ma: T[], f: (value: T) => U[]): U[] {
  const result: U[] = [];
  for (const value of ma) {
    result.push(...f(value));
  }
  return result;
}

// List monadic functions
function mapList<T, U>(fa: List<T>, f: (a: T) => U): List<U> {
  if (fa.tag === 'Empty') return { tag: 'Empty' };
  return { tag: 'Cons', head: f(fa.head), tail: mapList(fa.tail, f) };
}

function applyList<T, U>(ff: List<(a: T) => U>, fa: List<T>): List<U> {
  if (ff.tag === 'Empty') return { tag: 'Empty' };
  const mappedValues = mapList(fa, ff.head);
  const restApplied = applyList(ff.tail, fa);
  return concatList(mappedValues, restApplied);
}

function concatList<T>(list1: List<T>, list2: List<T>): List<T> {
  if (list1.tag === 'Empty') return list2;
  return { tag: 'Cons', head: list1.head, tail: concatList(list1.tail, list2) };
}

function bindList<T, U>(ma: List<T>, f: (value: T) => List<U>): List<U> {
  if (ma.tag === 'Empty') return { tag: 'Empty' };
  const headResult = f(ma.head);
  const tailResult = bindList(ma.tail, f);
  return concatList(headResult, tailResult);
}

// Maybe monadic functions
function mapMaybe<T, U>(fa: Maybe<T>, f: (a: T) => U): Maybe<U> {
  return fa.tag === 'Just' ? Just(f(fa.value)) : Nothing;
}

function applyMaybe<T, U>(ff: Maybe<(a: T) => U>, fa: Maybe<T>): Maybe<U> {
  return ff.tag === 'Just' && fa.tag === 'Just' ? Just(ff.value(fa.value)) : Nothing;
}

function bindMaybe<T, U>(ma: Maybe<T>, f: (value: T) => Maybe<U>): Maybe<U> {
  return ma.tag === 'Just' ? f(ma.value) : Nothing;
}

// Either monadic functions
function mapEither<L, R, U>(ea: Either<L, R>, f: (value: R) => U): Either<L, U> {
  return ea.tag === 'Right' ? Right(f(ea.value)) : ea;
}

function applyEither<L, R, U>(ef: Either<L, (value: R) => U>, ea: Either<L, R>): Either<L, U> {
  return ef.tag === 'Right' && ea.tag === 'Right' ? Right(ef.value(ea.value)) :
         ef.tag === 'Left' ? ef : ea as Either<L, U>;
}

function bindEither<L, R, U>(ea: Either<L, R>, f: (value: R) => Either<L, U>): Either<L, U> {
  return ea.tag === 'Right' ? f(ea.value) : ea;
}

const Unit: Unit = { tag: 'Unit', value: undefined };

function Just<T>(value: T): Maybe<T> { return { tag: 'Just', value }; }
const Nothing: Maybe<never> = { tag: 'Nothing' };

function Left<L>(value: L): Either<L, never> { return { tag: 'Left', value }; }
function Right<R>(value: R): Either<never, R> { return { tag: 'Right', value }; }

// Nullish coalescing helper functions
function fromMaybe<T>(defaultValue: T, maybe: Maybe<T>): T {
  return maybe.tag === 'Just' ? maybe.value : defaultValue;
}

function fromRight<L, R>(defaultValue: R, either: Either<L, R>): R {
  return either.tag === 'Right' ? either.value : defaultValue;
}

function fromLeft<L, R>(defaultValue: L, either: Either<L, R>): L {
  return either.tag === 'Left' ? either.value : defaultValue;
}

const Empty: List<never> = { tag: 'Empty' };
function Cons<T>(head: T, tail: List<T>): List<T> { return { tag: 'Cons', head, tail }; }

function headList<T>(list: List<T>): Maybe<T> { return list.tag === 'Cons' ? { tag: 'Just', value: list.head } : { tag: 'Nothing' }; }
function tailList<T>(list: List<T>): List<T> { return list.tag === 'Cons' ? list.tail : Empty; }

function ssrgPrint(value: unknown): void {
  // Seseragi型の場合は美しく整形
  if (value && typeof value === 'object' && (
    (value as any).tag === 'Unit' ||
    (value as any).tag === 'Just' || (value as any).tag === 'Nothing' ||
    (value as any).tag === 'Left' || (value as any).tag === 'Right' ||
    (value as any).tag === 'Cons' || (value as any).tag === 'Empty'
  )) {
    console.log(ssrgToString(value))
  }
  // 通常のオブジェクトはそのまま
  else {
    console.log(value)
  }
}
function ssrgPutStrLn(value: string): void { console.log(value); }
function ssrgToString(value: unknown): string {
  // Unit型の美しい表示
  if (value && typeof value === 'object' && (value as any).tag === 'Unit') {
    return '()'
  }

  // Maybe型の美しい表示
  if (value && typeof value === 'object' && (value as any).tag === 'Just') {
    return `Just(${ssrgToString((value as any).value)})`
  }
  if (value && typeof value === 'object' && (value as any).tag === 'Nothing') {
    return 'Nothing'
  }

  // Either型の美しい表示
  if (value && typeof value === 'object' && (value as any).tag === 'Left') {
    return `Left(${ssrgToString((value as any).value)})`
  }
  if (value && typeof value === 'object' && (value as any).tag === 'Right') {
    return `Right(${ssrgToString((value as any).value)})`
  }

  // List型の美しい表示
  if (value && typeof value === 'object' && (value as any).tag === 'Empty') {
    return "`[]"
  }
  if (value && typeof value === 'object' && (value as any).tag === 'Cons') {
    const items: string[] = []
    let current = value as any
    while (current.tag === 'Cons') {
      items.push(ssrgToString(current.head))
      current = current.tail
    }
    return "`[" + items.join(', ') + "]"
  }

  // Tuple型の美しい表示
  if (value && typeof value === 'object' && (value as any).tag === 'Tuple') {
    return `(${(value as any).elements.map(ssrgToString).join(', ')})`
  }

  // 配列の表示
  if (Array.isArray(value)) {
    return `[${value.map(ssrgToString).join(', ')}]`
  }

  // プリミティブ型
  if (typeof value === 'string') {
    return `"${value}"`
  }
  if (typeof value === 'number') {
    return String(value)
  }
  if (typeof value === 'boolean') {
    return value ? 'True' : 'False'
  }

  // 普通のオブジェクト（構造体など）
  if (typeof value === 'object' && value !== null) {
    const pairs: string[] = []
    for (const key in value) {
      if ((value as any).hasOwnProperty(key)) {
        pairs.push(`${key}: ${ssrgToString((value as any)[key])}`)
      }
    }

    // 構造体名を取得（constructor.nameを使用）
    const structName = (value as any).constructor && (value as any).constructor.name !== 'Object'
      ? (value as any).constructor.name
      : ''

    // 複数フィールドがある場合はインデント表示
    if (pairs.length > 2) {
      return `${structName} {\n  ${pairs.join(',\n  ')}\n}`
    } else {
      return `${structName} { ${pairs.join(', ')} }`
    }
  }

  return String(value)
}
function ssrgToInt(value: unknown): number {
  if (typeof value === 'number') {
    return Math.trunc(value)
  }
  if (typeof value === 'string') {
    const n = parseInt(value, 10)
    if (isNaN(n)) {
      throw new Error(`Cannot convert "${value}" to Int`)
    }
    return n
  }
  throw new Error(`Cannot convert ${typeof value} to Int`)
}
function ssrgToFloat(value: unknown): number {
  if (typeof value === 'number') {
    return value
  }
  if (typeof value === 'string') {
    const n = parseFloat(value)
    if (isNaN(n)) {
      throw new Error(`Cannot convert "${value}" to Float`)
    }
    return n
  }
  throw new Error(`Cannot convert ${typeof value} to Float`)
}
function ssrgShow(value: unknown): void {
  console.log(ssrgToString(value))
}
function ssrgTypeOf(value: unknown, variableName?: string): string {
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
    if ((value as any).tag === "Tuple" && Array.isArray((value as any).elements)) {
      const elemTypes = (value as any).elements.map((elem: any) => ssrgTypeOf(elem))
      return `(${elemTypes.join(', ')})`
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
    if ((value as any).constructor && (value as any).constructor.name !== 'Object') {
      return (value as any).constructor.name
    }
    
    // 5. 匿名オブジェクトの場合は型構造を返す（構造的型システム）
    const keys = Object.keys(value as any).sort() // キーをソートして順序を統一
    if (keys.length > 0) {
      const fields = keys.map(key => `${key}: ${ssrgTypeOf((value as any)[key])}`).join(', ')
      return `{ ${fields} }`
    }
  }
  
  return "unknown"
}
function ssrgTypeOfWithAliases(value: unknown, variableName?: string): string {
  // 構造的型を取得（変数テーブルを使わずに）
  const structuralType = ssrgTypeOf(value)  // 変数名なしで呼ぶ
  
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
    return structuralType + " (" + matchingAliases.join(', ') + ")"
  }
  
  return structuralType
}

function getStructuralTypeString(value: any): string {
  if (!value || typeof value !== "object") return "unknown"
  const keys = Object.keys(value).sort() // キーをソートして順序を統一
  if (keys.length === 0) return "{}"
  const fields = keys.map(key => `${key}: ${ssrgTypeOf(value[key])}`).join(', ')
  return `{ ${fields} }`
}

function typeMatches(structuralType: string, typeInfo: any): boolean {
  if (!typeInfo || typeof typeInfo !== "object") return false
  
  switch (typeInfo.kind) {
    case "record":
      const expectedFields = Object.keys(typeInfo.fields)
        .sort() // キーをソートして順序を統一
        .map(name => `${name}: ${getTypeInfoString(typeInfo.fields[name])}`)
        .join(', ')
      return structuralType === `{ ${expectedFields} }`
    case "tuple":
      const expectedElements = typeInfo.elements
        .map((elem: any) => getTypeInfoString(elem))
        .join(', ')
      return structuralType === `(${expectedElements})`
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
      return `(${typeInfo.elements.map((elem: any) => getTypeInfoString(elem)).join(', ')})`
    case "record":
      const fields = Object.keys(typeInfo.fields)
        .map(name => `${name}: ${getTypeInfoString(typeInfo.fields[name])}`)
        .join(', ')
      return `{ ${fields} }`
    default:
      return "unknown"
  }
}

function wildcardTypeMatches(actualType: string, expectedType: string): boolean {
  // ワイルドカード型マッチング：Either<String, Int> と Either<String, _> などをマッチ
  if (expectedType.startsWith("Either<")) {
    if (!actualType.startsWith("Either<")) return false
    
    const actualContent = actualType.slice(7, -1)
    const expectedContent = expectedType.slice(7, -1)
    
    const actualCommaIndex = actualContent.indexOf(',')
    const expectedCommaIndex = expectedContent.indexOf(',')
    
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
    actualContent.split(',').forEach(field => {
      const colonIndex = field.indexOf(':')
      if (colonIndex !== -1) {
        const name = field.slice(0, colonIndex).trim()
        const type = field.slice(colonIndex + 1).trim()
        actualFields[name] = type
      }
    })
    
    // expectedTypeをパース
    expectedContent.split(',').forEach(field => {
      const colonIndex = field.indexOf(':')
      if (colonIndex !== -1) {
        const name = field.slice(0, colonIndex).trim()
        const type = field.slice(colonIndex + 1).trim()
        expectedFields[name] = type
      }
    })
    
    // フィールド数が一致する必要がある
    if (Object.keys(actualFields).length !== Object.keys(expectedFields).length) return false
    
    // 各フィールドをチェック
    for (const [fieldName, expectedFieldType] of Object.entries(expectedFields)) {
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
    
    const actualElements = actualContent.split(',').map(e => e.trim())
    const expectedElements = expectedContent.split(',').map(e => e.trim())
    
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
function ssrgIsType(value: unknown, typeString: string, variableName?: string): boolean {
  // 1. 変数型テーブルからの型情報チェック（ワイルドカードがない場合のみ最優先）
  if (variableName && __variableTypes[variableName] && !typeString.includes('_')) {
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
    if (variableType.startsWith("{") && variableType.endsWith("}") && 
        typeString.startsWith("{") && typeString.endsWith("}")) {
      return checkStructuralType(value, typeString)
    }
    
    return false
  }
  
  // 2. ワイルドカード型の場合は、変数型テーブルの情報とワイルドカードマッチングを組み合わせる
  if (variableName && __variableTypes[variableName] && typeString.includes('_')) {
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
        const commaIndex = content.indexOf(',')
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
          return value.every(item => ssrgIsType(item, elemType))
        }
      }
      return false
    }
    
    // Tuple型チェック
    if (typeString.startsWith("(") && typeString.endsWith(")")) {
      if ((value as any).tag === "Tuple" && Array.isArray((value as any).elements)) {
        const tupleContent = typeString.slice(1, -1)
        const expectedTypes = tupleContent.split(',').map(t => t.trim())
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

function typeMatchesRegistry(value: any, typeInfo: any): boolean {
  if (!typeInfo || typeof typeInfo !== "object") return false
  
  switch (typeInfo.kind) {
    case "primitive":
      return ssrgTypeOf(value) === typeInfo.name
    case "record":
      if (!value || typeof value !== "object") return false
      return Object.keys(typeInfo.fields).every(fieldName => {
        if (!(fieldName in value)) return false
        return typeMatchesRegistry(value[fieldName], typeInfo.fields[fieldName])
      })
    case "tuple":
      if (!value || typeof value !== "object" || (value as any).tag !== "Tuple") return false
      const elements = (value as any).elements
      if (!Array.isArray(elements) || elements.length !== typeInfo.elements.length) return false
      return typeInfo.elements.every((expectedType: any, index: number) => 
        typeMatchesRegistry(elements[index], expectedType)
      )
    case "array":
      if (!Array.isArray(value)) return false
      return value.every(item => typeMatchesRegistry(item, typeInfo.elementType))
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
  
  const fields = content.split(',').map(f => f.trim())
  const expectedFields: Record<string, string> = {}
  
  for (const field of fields) {
    const colonIndex = field.indexOf(':')
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
  const actualFieldCount = Object.keys(value).filter(key => key !== "__typename").length
  const expectedFieldCount = Object.keys(expectedFields).length
  return actualFieldCount === expectedFieldCount
}

// 型レジストリ（コンパイル時型情報の実行時保持）
const __typeRegistry: Record<string, any> = {};

// 変数型情報テーブル（完全型情報保持）
const __variableTypes: Record<string, string> = {};
const __variableAliases: Record<string, string[]> = {};

function arrayToList<T>(arr: T[]): List<T> {
  let result: List<T> = Empty;
  for (let i = arr.length - 1; i >= 0; i--) {
    result = Cons(arr[i], result);
  }
  return result;
}

function listToArray<T>(list: List<T>): T[] {
  const result: T[] = [];
  let current = list;
  while (current.tag === 'Cons') {
    result.push(current.head);
    current = current.tail;
  }
  return result;
}

// Struct method and operator dispatch tables
let __structMethods: Record<string, Record<string, Function>> = {};
let __structOperators: Record<string, Record<string, Function>> = {};

// Method dispatch helper
function __dispatchMethod(obj: any, methodName: string, ...args: any[]): any {
  // 構造体のフィールドアクセスの場合は直接返す
  if (args.length === 0 && obj.hasOwnProperty(methodName)) {
    return obj[methodName];
  }
  const structName = obj.constructor.name;
  const structMethods = __structMethods[structName];
  if (structMethods && structMethods[methodName]) {
    return structMethods[methodName](obj, ...args);
  }
  throw new Error(`Method '${methodName}' not found for struct '${structName}'`);
}

// Operator dispatch helper
function __dispatchOperator(left: any, operator: string, right: any): any {
  const structName = left.constructor.name;
  const structOperators = __structOperators[structName];
  if (structOperators && structOperators[operator]) {
    return structOperators[operator](left, right);
  }
  // Fall back to native JavaScript operator
  switch (operator) {
    case '+': return left + right;
    case '-': return left - right;
    case '*': return left * right;
    case '/': return left / right;
    case '%': return left % right;
    case '**': return left ** right;
    case '==': return left == right;
    case '!=': return left != right;
    case '<': return left < right;
    case '>': return left > right;
    case '<=': return left <= right;
    case '>=': return left >= right;
    case '&&': return left && right;
    case '||': return left || right;
    default: throw new Error(`Unknown operator: ${operator}`);
  }
}


// 変数型情報テーブルの初期化
Object.assign(__variableTypes, {
  "maybeValue": "Maybe<Int>",
  "nothingValue": "Maybe<unknown>",
  "doubled": "Maybe<unknown>",
  "doubled'": "Maybe<unknown>",
  "rightValue": "Either<unknown, Int>",
  "leftValue": "Either<String, unknown>",
  "incremented": "Either<unknown, unknown>",
  "incremented'": "Either<String, unknown>",
  "value1": "Maybe<Int>",
  "value2": "Maybe<Int>",
  "value3": "Maybe<unknown>",
  "sum": "Maybe<unknown>",
  "sum'": "Maybe<unknown>",
  "right1": "Either<unknown, Int>",
  "right2": "Either<unknown, Int>",
  "left1": "Either<String, unknown>",
  "product": "Either<unknown, unknown>",
  "product'": "Either<unknown, unknown>",
  "result1": "Maybe<unknown>",
  "result2": "Maybe<unknown>",
  "chain1": "Maybe<unknown>",
  "chain2": "Maybe<unknown>",
  "valid1": "Either<unknown, unknown>",
  "valid2": "Either<unknown, unknown>",
  "valid3": "Either<unknown, unknown>",
  "arr": "Array<Int>",
  "mapped": "Array<unknown>",
  "lst": "List<Int>",
  "transformed": "List<unknown>",
  "flattened": "unknown",
  "listFlattened": "List<unknown>",
  "x'": "Maybe<Int>",
  "y'": "Maybe<Int>",
  "result'": "Maybe<unknown>",
  "value": "Maybe<Int>",
  "value'": "Maybe<unknown>",
  "nameValue": "Maybe<String>",
  "domainValue": "Maybe<String>",
  "email": "Maybe<unknown>",
  "result": "Maybe<unknown>"
});

ssrgPrint("=== モナド演算子の基本 ===");

ssrgPrint("--- ファンクター演算子 <$> ---");

function double(x: number): number {
  return (x * 2);
}

function increment(x: number): number {
  return (x + 1);
}

const maybeValue = Just(42);

const nothingValue = Nothing;

const doubled = (() => {
      const _value = maybeValue;
      if (_value && _value.tag === 'Tuple') {
        return { tag: 'Tuple', elements: mapArray(_value.elements, double) };
      } else if (Array.isArray(_value)) {
        return mapArray(_value, double);
      } else if (_value && _value.tag === 'Cons' || _value && _value.tag === 'Empty') {
        return mapList(_value, double);
      } else if (_value && (_value.tag === 'Left' || _value.tag === 'Right')) {
        return mapEither(_value, double);
      } else {
        return mapMaybe(_value, double);
      }
    })();

const doubled_prime = (() => {
      const _value = nothingValue;
      if (_value && _value.tag === 'Tuple') {
        return { tag: 'Tuple', elements: mapArray(_value.elements, double) };
      } else if (Array.isArray(_value)) {
        return mapArray(_value, double);
      } else if (_value && _value.tag === 'Cons' || _value && _value.tag === 'Empty') {
        return mapList(_value, double);
      } else if (_value && (_value.tag === 'Left' || _value.tag === 'Right')) {
        return mapEither(_value, double);
      } else {
        return mapMaybe(_value, double);
      }
    })();

ssrgShow(doubled);

ssrgShow(doubled_prime);

const rightValue = Right(10);

const leftValue = Left("error");

const incremented = (() => {
      const _value = rightValue;
      if (_value && _value.tag === 'Tuple') {
        return { tag: 'Tuple', elements: mapArray(_value.elements, increment) };
      } else if (Array.isArray(_value)) {
        return mapArray(_value, increment);
      } else if (_value && _value.tag === 'Cons' || _value && _value.tag === 'Empty') {
        return mapList(_value, increment);
      } else if (_value && (_value.tag === 'Left' || _value.tag === 'Right')) {
        return mapEither(_value, increment);
      } else {
        return mapMaybe(_value, increment);
      }
    })();

const incremented_prime = (() => {
      const _value = leftValue;
      if (_value && _value.tag === 'Tuple') {
        return { tag: 'Tuple', elements: mapArray(_value.elements, increment) };
      } else if (Array.isArray(_value)) {
        return mapArray(_value, increment);
      } else if (_value && _value.tag === 'Cons' || _value && _value.tag === 'Empty') {
        return mapList(_value, increment);
      } else if (_value && (_value.tag === 'Left' || _value.tag === 'Right')) {
        return mapEither(_value, increment);
      } else {
        return mapMaybe(_value, increment);
      }
    })();

ssrgShow(incremented);

ssrgShow(incremented_prime);

ssrgPrint("--- アプリカティブ演算子 <*> ---");

function add(x: number): (arg: number) => number {
  return function(y: number) {
      return (x + y);
    };
}

function multiply(x: number): (arg: number) => number {
  return function(y: number) {
      return (x * y);
    };
}

const value1 = Just(10);

const value2 = Just(5);

const value3 = Nothing;

const sum = (() => {
      const _funcs = (() => {
      const _funcs = Just(add);
      const _values = value1;
      if (Array.isArray(_funcs) && Array.isArray(_values)) {
        return applyArray(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Cons' || _funcs.tag === 'Empty') &&
                _values && (_values.tag === 'Cons' || _values.tag === 'Empty')) {
        return applyList(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Left' || _funcs.tag === 'Right') &&
                _values && (_values.tag === 'Left' || _values.tag === 'Right')) {
        return applyEither(_funcs, _values);
      } else {
        return applyMaybe(_funcs, _values);
      }
    })();
      const _values = value2;
      if (Array.isArray(_funcs) && Array.isArray(_values)) {
        return applyArray(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Cons' || _funcs.tag === 'Empty') &&
                _values && (_values.tag === 'Cons' || _values.tag === 'Empty')) {
        return applyList(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Left' || _funcs.tag === 'Right') &&
                _values && (_values.tag === 'Left' || _values.tag === 'Right')) {
        return applyEither(_funcs, _values);
      } else {
        return applyMaybe(_funcs, _values);
      }
    })();

const sum_prime = (() => {
      const _funcs = (() => {
      const _funcs = Just(add);
      const _values = value1;
      if (Array.isArray(_funcs) && Array.isArray(_values)) {
        return applyArray(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Cons' || _funcs.tag === 'Empty') &&
                _values && (_values.tag === 'Cons' || _values.tag === 'Empty')) {
        return applyList(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Left' || _funcs.tag === 'Right') &&
                _values && (_values.tag === 'Left' || _values.tag === 'Right')) {
        return applyEither(_funcs, _values);
      } else {
        return applyMaybe(_funcs, _values);
      }
    })();
      const _values = value3;
      if (Array.isArray(_funcs) && Array.isArray(_values)) {
        return applyArray(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Cons' || _funcs.tag === 'Empty') &&
                _values && (_values.tag === 'Cons' || _values.tag === 'Empty')) {
        return applyList(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Left' || _funcs.tag === 'Right') &&
                _values && (_values.tag === 'Left' || _values.tag === 'Right')) {
        return applyEither(_funcs, _values);
      } else {
        return applyMaybe(_funcs, _values);
      }
    })();

ssrgShow(sum);

ssrgShow(sum_prime);

const right1 = Right(20);

const right2 = Right(3);

const left1 = Left("first error");

const product = (() => {
      const _funcs = (() => {
      const _funcs = Right(multiply);
      const _values = right1;
      if (Array.isArray(_funcs) && Array.isArray(_values)) {
        return applyArray(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Cons' || _funcs.tag === 'Empty') &&
                _values && (_values.tag === 'Cons' || _values.tag === 'Empty')) {
        return applyList(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Left' || _funcs.tag === 'Right') &&
                _values && (_values.tag === 'Left' || _values.tag === 'Right')) {
        return applyEither(_funcs, _values);
      } else {
        return applyMaybe(_funcs, _values);
      }
    })();
      const _values = right2;
      if (Array.isArray(_funcs) && Array.isArray(_values)) {
        return applyArray(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Cons' || _funcs.tag === 'Empty') &&
                _values && (_values.tag === 'Cons' || _values.tag === 'Empty')) {
        return applyList(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Left' || _funcs.tag === 'Right') &&
                _values && (_values.tag === 'Left' || _values.tag === 'Right')) {
        return applyEither(_funcs, _values);
      } else {
        return applyMaybe(_funcs, _values);
      }
    })();

const product_prime = (() => {
      const _funcs = (() => {
      const _funcs = Right(multiply);
      const _values = left1;
      if (Array.isArray(_funcs) && Array.isArray(_values)) {
        return applyArray(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Cons' || _funcs.tag === 'Empty') &&
                _values && (_values.tag === 'Cons' || _values.tag === 'Empty')) {
        return applyList(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Left' || _funcs.tag === 'Right') &&
                _values && (_values.tag === 'Left' || _values.tag === 'Right')) {
        return applyEither(_funcs, _values);
      } else {
        return applyMaybe(_funcs, _values);
      }
    })();
      const _values = right2;
      if (Array.isArray(_funcs) && Array.isArray(_values)) {
        return applyArray(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Cons' || _funcs.tag === 'Empty') &&
                _values && (_values.tag === 'Cons' || _values.tag === 'Empty')) {
        return applyList(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Left' || _funcs.tag === 'Right') &&
                _values && (_values.tag === 'Left' || _values.tag === 'Right')) {
        return applyEither(_funcs, _values);
      } else {
        return applyMaybe(_funcs, _values);
      }
    })();

ssrgShow(product);

ssrgShow(product_prime);

ssrgPrint("--- モナド演算子 >>= ---");

function doubleIfEven(x: number): Maybe<number> {
  return (((x % 2) === 0) ? Just((x * 2)) : Nothing);
}

function addTen(x: number): Maybe<number> {
  return ((x > 0) ? Just((x + 10)) : Nothing);
}

const result1 = (() => {
      const _monad = Just(4);
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, doubleIfEven);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, doubleIfEven);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, doubleIfEven);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, doubleIfEven);
      } else {
        return bindMaybe(_monad, doubleIfEven);
      }
    })();

const result2 = (() => {
      const _monad = Just(3);
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, doubleIfEven);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, doubleIfEven);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, doubleIfEven);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, doubleIfEven);
      } else {
        return bindMaybe(_monad, doubleIfEven);
      }
    })();

ssrgShow(result1);

ssrgShow(result2);

const chain1 = (() => {
      const _monad = (() => {
      const _monad = Just(2);
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, doubleIfEven);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, doubleIfEven);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, doubleIfEven);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, doubleIfEven);
      } else {
        return bindMaybe(_monad, doubleIfEven);
      }
    })();
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, addTen);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, addTen);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, addTen);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, addTen);
      } else {
        return bindMaybe(_monad, addTen);
      }
    })();

const chain2 = (() => {
      const _monad = (() => {
      const _monad = Just(3);
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, doubleIfEven);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, doubleIfEven);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, doubleIfEven);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, doubleIfEven);
      } else {
        return bindMaybe(_monad, doubleIfEven);
      }
    })();
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, addTen);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, addTen);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, addTen);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, addTen);
      } else {
        return bindMaybe(_monad, addTen);
      }
    })();

ssrgShow(chain1);

ssrgShow(chain2);

function validatePositive(x: number): Either<string, number> {
  return ((x > 0) ? Right(x) : Left("値は正数である必要があります"));
}

function validateEven(x: number): Either<string, number> {
  return (((x % 2) === 0) ? Right(x) : Left("値は偶数である必要があります"));
}

const valid1 = (() => {
      const _monad = (() => {
      const _monad = Right(8);
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, validatePositive);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, validatePositive);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, validatePositive);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, validatePositive);
      } else {
        return bindMaybe(_monad, validatePositive);
      }
    })();
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, validateEven);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, validateEven);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, validateEven);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, validateEven);
      } else {
        return bindMaybe(_monad, validateEven);
      }
    })();

const valid2 = (() => {
      const _monad = (() => {
      const _monad = Right(7);
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, validatePositive);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, validatePositive);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, validatePositive);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, validatePositive);
      } else {
        return bindMaybe(_monad, validatePositive);
      }
    })();
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, validateEven);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, validateEven);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, validateEven);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, validateEven);
      } else {
        return bindMaybe(_monad, validateEven);
      }
    })();

const valid3 = (() => {
      const _monad = (() => {
      const _monad = Right(-2);
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, validatePositive);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, validatePositive);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, validatePositive);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, validatePositive);
      } else {
        return bindMaybe(_monad, validatePositive);
      }
    })();
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, validateEven);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, validateEven);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, validateEven);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, validateEven);
      } else {
        return bindMaybe(_monad, validateEven);
      }
    })();

ssrgShow(valid1);

ssrgShow(valid2);

ssrgShow(valid3);

const arr = [1, 2, 3];

const mapped = (() => {
      const _value = arr;
      if (_value && _value.tag === 'Tuple') {
        return { tag: 'Tuple', elements: mapArray(_value.elements, (x: any) => (x * 2)) };
      } else if (Array.isArray(_value)) {
        return mapArray(_value, (x: any) => (x * 2));
      } else if (_value && _value.tag === 'Cons' || _value && _value.tag === 'Empty') {
        return mapList(_value, (x: any) => (x * 2));
      } else if (_value && (_value.tag === 'Left' || _value.tag === 'Right')) {
        return mapEither(_value, (x: any) => (x * 2));
      } else {
        return mapMaybe(_value, (x: any) => (x * 2));
      }
    })();

ssrgShow(mapped);

const lst = Cons(1, Cons(2, Cons(3, Empty)));

const transformed = (() => {
      const _value = lst;
      if (_value && _value.tag === 'Tuple') {
        return { tag: 'Tuple', elements: mapArray(_value.elements, (x: any) => (x * 3)) };
      } else if (Array.isArray(_value)) {
        return mapArray(_value, (x: any) => (x * 3));
      } else if (_value && _value.tag === 'Cons' || _value && _value.tag === 'Empty') {
        return mapList(_value, (x: any) => (x * 3));
      } else if (_value && (_value.tag === 'Left' || _value.tag === 'Right')) {
        return mapEither(_value, (x: any) => (x * 3));
      } else {
        return mapMaybe(_value, (x: any) => (x * 3));
      }
    })();

ssrgShow(transformed);

function repeat(x: number): Array<number> {
  return [x, x];
}

const flattened = (() => {
      const _monad = arr;
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, repeat);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, repeat);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, repeat);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, repeat);
      } else {
        return bindMaybe(_monad, repeat);
      }
    })();

ssrgShow(flattened);

function listRepeat(x: number): List<number> {
  return Cons(x, Cons(x, Empty));
}

const listFlattened = (() => {
      const _monad = lst;
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, listRepeat);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, listRepeat);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, listRepeat);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, listRepeat);
      } else {
        return bindMaybe(_monad, listRepeat);
      }
    })();

ssrgShow(listFlattened);

ssrgPrint("--- 実用的な例 ---");

const x_prime = Just(100);

const y_prime = Just(25);

function subtract(x: number): (arg: number) => number {
  return function(y: number) {
      return (x - y);
    };
}

const result_prime = (() => {
      const _funcs = (() => {
      const _funcs = Just(subtract);
      const _values = x_prime;
      if (Array.isArray(_funcs) && Array.isArray(_values)) {
        return applyArray(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Cons' || _funcs.tag === 'Empty') &&
                _values && (_values.tag === 'Cons' || _values.tag === 'Empty')) {
        return applyList(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Left' || _funcs.tag === 'Right') &&
                _values && (_values.tag === 'Left' || _values.tag === 'Right')) {
        return applyEither(_funcs, _values);
      } else {
        return applyMaybe(_funcs, _values);
      }
    })();
      const _values = y_prime;
      if (Array.isArray(_funcs) && Array.isArray(_values)) {
        return applyArray(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Cons' || _funcs.tag === 'Empty') &&
                _values && (_values.tag === 'Cons' || _values.tag === 'Empty')) {
        return applyList(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Left' || _funcs.tag === 'Right') &&
                _values && (_values.tag === 'Left' || _values.tag === 'Right')) {
        return applyEither(_funcs, _values);
      } else {
        return applyMaybe(_funcs, _values);
      }
    })();

ssrgShow(result_prime);

const value = Just(100);

const value_prime = (() => {
      const _value = value;
      if (_value && _value.tag === 'Tuple') {
        return { tag: 'Tuple', elements: mapArray(_value.elements, toString) };
      } else if (Array.isArray(_value)) {
        return mapArray(_value, toString);
      } else if (_value && _value.tag === 'Cons' || _value && _value.tag === 'Empty') {
        return mapList(_value, toString);
      } else if (_value && (_value.tag === 'Left' || _value.tag === 'Right')) {
        return mapEither(_value, toString);
      } else {
        return mapMaybe(_value, toString);
      }
    })();

ssrgShow(value_prime);

function createEmail(name: string): (arg: string) => string {
  return function(domain: string) {
      return `${name}@${domain}`;
    };
}

const nameValue = Just("user");

const domainValue = Just("example.com");

const email = (() => {
      const _funcs = (() => {
      const _funcs = Just(createEmail);
      const _values = nameValue;
      if (Array.isArray(_funcs) && Array.isArray(_values)) {
        return applyArray(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Cons' || _funcs.tag === 'Empty') &&
                _values && (_values.tag === 'Cons' || _values.tag === 'Empty')) {
        return applyList(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Left' || _funcs.tag === 'Right') &&
                _values && (_values.tag === 'Left' || _values.tag === 'Right')) {
        return applyEither(_funcs, _values);
      } else {
        return applyMaybe(_funcs, _values);
      }
    })();
      const _values = domainValue;
      if (Array.isArray(_funcs) && Array.isArray(_values)) {
        return applyArray(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Cons' || _funcs.tag === 'Empty') &&
                _values && (_values.tag === 'Cons' || _values.tag === 'Empty')) {
        return applyList(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Left' || _funcs.tag === 'Right') &&
                _values && (_values.tag === 'Left' || _values.tag === 'Right')) {
        return applyEither(_funcs, _values);
      } else {
        return applyMaybe(_funcs, _values);
      }
    })();

ssrgShow(email);

function parsePort(input: string): Maybe<number> {
  return ((input === "80") ? Just(80) : ((input === "443") ? Just(443) : Nothing));
}

function validatePort(port: number): Maybe<string> {
  return ((port === 80) ? Just("HTTP") : ((port === 443) ? Just("HTTPS") : Nothing));
}

const result = (() => {
      const _monad = (() => {
      const _monad = Just("80");
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, parsePort);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, parsePort);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, parsePort);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, parsePort);
      } else {
        return bindMaybe(_monad, parsePort);
      }
    })();
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, validatePort);
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, validatePort);
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, validatePort);
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, validatePort);
      } else {
        return bindMaybe(_monad, validatePort);
      }
    })();

ssrgShow(result);

ssrgPrint("--- 演算子の利点 ---");

ssrgPrint("モナド演算子により、安全で表現力豊かなコードが書けます");

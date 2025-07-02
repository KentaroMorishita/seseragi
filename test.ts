// Generated TypeScript code from Seseragi

// Seseragi minimal runtime

type Maybe<T> = { tag: 'Just'; value: T } | { tag: 'Nothing' };
const Just = <T>(value: T): Maybe<T> => ({ tag: 'Just', value });
const Nothing: Maybe<never> = { tag: 'Nothing' };

const toString = (value: any): string => {
  // Maybe型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Just') {
    return `Just(${toString(value.value)})`
  }
  if (value && typeof value === 'object' && value.tag === 'Nothing') {
    return 'Nothing'
  }
  
  // Either型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Left') {
    return `Left(${toString(value.value)})`
  }
  if (value && typeof value === 'object' && value.tag === 'Right') {
    return `Right(${toString(value.value)})`
  }
  
  // List型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Empty') {
    return "`[]"
  }
  if (value && typeof value === 'object' && value.tag === 'Cons') {
    const items = []
    let current = value
    while (current.tag === 'Cons') {
      items.push(toString(current.head))
      current = current.tail
    }
    return "`[" + items.join(', ') + "]"
  }
  
  // 配列の表示
  if (Array.isArray(value)) {
    return `[${value.map(toString).join(', ')}]`
  }
  
  // プリミティブ型
  if (typeof value === 'string') {
    return `"${value}"`
  }
  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value)
  }
  
  // 普通のオブジェクト（構造体など）
  if (typeof value === 'object' && value !== null) {
    const pairs = []
    for (const key in value) {
      if (value.hasOwnProperty(key)) {
        pairs.push(`${key}: ${toString(value[key])}`)
      }
    }
    
    // 構造体名を取得（constructor.nameを使用）
    const structName = value.constructor && value.constructor.name !== 'Object' 
      ? value.constructor.name 
      : ''
    
    // 複数フィールドがある場合はインデント表示
    if (pairs.length > 2) {
      return `${structName} {\n  ${pairs.join(',\n  ')}\n}`
    } else {
      return `${structName} { ${pairs.join(', ')} }`
    }
  }
  
  return String(value)
};
// Seseragi型の構造を正規化
function normalizeStructure(obj) {
  if (!obj || typeof obj !== 'object') return obj
  
  // List型 → 特別なマーカー付き配列に変換
  if (obj.tag === 'Empty') return { '@@type': 'List', value: [] }
  if (obj.tag === 'Cons') {
    const items = []
    let current = obj
    while (current && current.tag === 'Cons') {
      items.push(normalizeStructure(current.head))
      current = current.tail
    }
    return { '@@type': 'List', value: items }
  }
  
  // Maybe型
  if (obj.tag === 'Just') {
    return { '@@type': 'Just', value: normalizeStructure(obj.value) }
  }
  if (obj.tag === 'Nothing') {
    return '@@Nothing'
  }
  
  // Either型
  if (obj.tag === 'Right') {
    return { '@@type': 'Right', value: normalizeStructure(obj.value) }
  }
  if (obj.tag === 'Left') {
    return { '@@type': 'Left', value: normalizeStructure(obj.value) }
  }
  
  // 配列
  if (Array.isArray(obj)) {
    return obj.map(normalizeStructure)
  }
  
  // 通常のオブジェクト
  const result = {}
  for (const key in obj) {
    if (obj.hasOwnProperty(key)) {
      result[key] = normalizeStructure(obj[key])
    }
  }
  return result
}

// JSON文字列をSeseragi型の美しい表記に変換
function beautifySeseragiTypes(json) {
  let result = json
  
  // Seseragi特殊型の変換
  result = beautifySpecialTypes(result)
  
  // 普通のオブジェクト（構造体など）の変換
  result = beautifyStructObjects(result)
  
  return result
}

// Seseragi特殊型（Maybe、Either、List）の美しい変換
function beautifySpecialTypes(json) {
  return json
    // List型
    .replace(/\{\s*"@@type":\s*"List",\s*"value":\s*\[\s*\]\s*\}/g, '`[]')
    .replace(/\{\s*"@@type":\s*"List",\s*"value":\s*\[([\s\S]*?)\]\s*\}/g, (match, content) => {
      const cleanContent = content.replace(/\s+/g, ' ').trim()
      return `\`[${cleanContent}]`
    })
    // Maybe型
    .replace(/"@@type":\s*"Just",\s*"value":\s*([^}]+)/g, (_, val) => `Just(${val.trim()})`)
    .replace(/\{\s*Just\(([^)]+)\)\s*\}/g, 'Just($1)')
    .replace(/"@@Nothing"/g, 'Nothing')
    // Either型
    .replace(/"@@type":\s*"Right",\s*"value":\s*([^}]+)/g, (_, val) => `Right(${val.trim()})`)
    .replace(/\{\s*Right\(([^)]+)\)\s*\}/g, 'Right($1)')
    .replace(/"@@type":\s*"Left",\s*"value":\s*([^}]+)/g, (_, val) => `Left(${val.trim()})`)
    .replace(/\{\s*Left\(([^)]+)\)\s*\}/g, 'Left($1)')
}

// 普通のオブジェクト（構造体）の美しい変換
function beautifyStructObjects(json) {
  return json.replace(/\{([\s\S]*?)\}/g, (match, content) => {
    // 既に変換済みのSeseragi型は除外
    if (match.includes('Just(') || match.includes('Right(') || match.includes('Left(') || match.includes('`[')) {
      return match
    }
    
    // フィールドを解析
    const fields = content.trim().split(',').filter(f => f.trim())
    const jsFields = fields.map(field => {
      const cleaned = field.trim().replace(/"(\w+)":/g, '$1:')
      return cleaned
    })
    
    // 複数フィールドの場合はインデント表示を保持、少数フィールドは1行
    if (jsFields.length > 2) {
      return `{\n  ${jsFields.join(',\n  ')}\n}`
    } else {
      return `{ ${jsFields.join(', ')} }`
    }
  })
}


// 美しくフォーマットする関数
const prettyFormat = (value) => {
  // プリミティブ型
  if (typeof value === 'string') return `"${value}"`
  if (typeof value === 'number' || typeof value === 'boolean') return String(value)
  if (value === null) return 'null'
  if (value === undefined) return 'undefined'
  
  // オブジェクトの場合
  if (typeof value === 'object') {
    // まず構造を正規化
    const normalized = normalizeStructure(value)
    // JSON.stringifyで整形
    const json = JSON.stringify(normalized, null, 2)
    // Seseragi型の表記に変換
    return beautifySeseragiTypes(json)
  }
  
  return String(value)
}

const show = (value) => {
  console.log(prettyFormat(value))
};

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


class Point {
  constructor(public x: number, public y: number) {}
}

type User = { name: string, age: number };

const name = "Alice";

const age = 25;

const user1 = { name, age };

const x = 10;

const point1 = (() => { const __tmpwa7rrv = { x: x, y: 20 }; return Object.assign(Object.create(Point.prototype), __tmpwa7rrv); })();

const user2 = { name, age: 30 };

const existing = new Point(5, 10);

const y = 20;

const point2 = (() => { const __tmpp8cjfo = { ...existing, y: y }; return Object.assign(Object.create(Point.prototype), __tmpp8cjfo); })();

show(point2);

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
  
  // Tuple型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Tuple') {
    return `(${value.elements.map(toString).join(', ')})`
  }
  
  // 配列の表示
  if (Array.isArray(value)) {
    return `[${value.map(toString).join(', ')}]`
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
  // プリミティブ型の処理
  if (typeof obj === 'boolean') {
    return { '@@type': 'Boolean', value: obj }
  }
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
  
  // Tuple型
  if (obj.tag === 'Tuple') {
    return { '@@type': 'Tuple', value: obj.elements.map(normalizeStructure) }
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
    // タプル型
    .replace(/\{\s*"@@type":\s*"Tuple",\s*"value":\s*\[([\s\S]*?)\]\s*\}/g, (match, content) => {
      const cleanContent = content.replace(/\s+/g, ' ').trim()
      return `(${cleanContent})`
    })
    // ブール値型
    .replace(/\{\s*"@@type":\s*"Boolean",\s*"value":\s*(true|false)\s*\}/g, (match, value) => {
      return value === 'true' ? 'True' : 'False'
    })
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
  if (typeof value === 'number') return String(value)
  if (typeof value === 'boolean') return value ? 'True' : 'False'
  if (value === null) return 'null'
  if (value === undefined) return 'undefined'
  
  // Seseragi特殊型とオブジェクトの場合
  if (value && typeof value === 'object') {
    // Maybe型
    if (value.tag === 'Just') {
      return `Just(${prettyFormat(value.value)})`
    }
    if (value.tag === 'Nothing') {
      return 'Nothing'
    }
    
    // Either型
    if (value.tag === 'Left') {
      return `Left(${prettyFormat(value.value)})`
    }
    if (value.tag === 'Right') {
      return `Right(${prettyFormat(value.value)})`
    }
    
    // List型
    if (value.tag === 'Empty') {
      return '`[]'
    }
    if (value.tag === 'Cons') {
      const items = []
      let current = value
      while (current.tag === 'Cons') {
        items.push(prettyFormat(current.head))
        current = current.tail
      }
      return `\`[${items.join(', ')}]`
    }
    
    // Tuple型
    if (value.tag === 'Tuple') {
      return `(${value.elements.map(prettyFormat).join(', ')})`
    }
    
    // 配列
    if (Array.isArray(value)) {
      return `[${value.map(prettyFormat).join(', ')}]`
    }
    
    // 構造体・普通のオブジェクト
    const pairs = []
    for (const key in value) {
      if (value.hasOwnProperty(key)) {
        pairs.push(`${key}: ${prettyFormat(value[key])}`)
      }
    }
    
    const structName = value.constructor && value.constructor.name !== 'Object' 
      ? value.constructor.name 
      : ''
    
    if (pairs.length > 2) {
      return `${structName} {\n  ${pairs.join(',\n  ')}\n}`
    } else {
      return `${structName} { ${pairs.join(', ')} }`
    }
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
  x: number;
  y: number;

  constructor(fields: { x: number, y: number }) {
    this.x = fields.x;
    this.y = fields.y;
  }
}

class User {
  name: string;
  age: number;
  amount: number;

  constructor(fields: { name: string, age: number, amount: number }) {
    this.name = fields.name;
    this.age = fields.age;
    this.amount = fields.amount;
  }
}

// Point implementation
function __ssrg_Point_f4pl4mu_op_add(self: Point, other: Point): Point {
  return (() => {
  const x = (self.x + other.x);
  const y = (self.y + other.y);
  return (() => { const __tmpik8sxa = { x: x, y: y }; return Object.assign(Object.create(Point.prototype), __tmpik8sxa); })();
})();
}

// User implementation
function __ssrg_User_f4pl4mu_op_add(self: User, other: User): number {
  return (() => {
  return (self.amount + other.amount);
})();
}
function __ssrg_User_f4pl4mu_op_mul(self: User, v: number): number {
  return (() => {
  return (self.amount * v);
})();
}

// Initialize dispatch tables immediately
(() => {
  // Initialize method dispatch table
  __structMethods = {
    "Point": {

    },
    "User": {

    },
  };

  // Initialize operator dispatch table
  __structOperators = {
    "Point": {
      "+": __ssrg_Point_f4pl4mu_op_add
    },
    "User": {
      "+": __ssrg_User_f4pl4mu_op_add,
      "*": __ssrg_User_f4pl4mu_op_mul
    },
  };

})();

const func = (x: any) => (y: any) => __dispatchOperator(x, "+", y);

const hoge = func(1)(2);

show(hoge);

const fuga = func("foo")("bar");

show(fuga);

const p1 = new Point({ x: 1, y: 2 });

const p2 = new Point({ x: 3, y: 4 });

const foo = func(p1)(p2);

show(foo);

const u1 = new User({ name: "foo", age: 20, amount: 10 });

const u2 = new User({ name: "bar", age: 20, amount: 20 });

const bar = (func(u1)(u2) as number);

show(bar);

const baz = __dispatchOperator(u1, "+", u2);

show(baz);

const quax = __dispatchOperator(u1, "*", 2);

show(quax);

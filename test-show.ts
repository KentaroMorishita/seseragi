// Generated TypeScript code from Seseragi

// Seseragi minimal runtime

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
    return '[]'
  }
  if (value && typeof value === 'object' && value.tag === 'Cons') {
    const items = []
    let current = value
    while (current.tag === 'Cons') {
      items.push(toString(current.head))
      current = current.tail
    }
    return `[${items.join(', ')}]`
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
  
  return String(value)
};
// Seseragi型の構造を正規化
function normalizeStructure(obj) {
  if (!obj || typeof obj !== 'object') return obj
  
  // List型 → 配列に変換
  if (obj.tag === 'Empty') return []
  if (obj.tag === 'Cons') {
    const items = []
    let current = obj
    while (current && current.tag === 'Cons') {
      items.push(normalizeStructure(current.head))
      current = current.tail
    }
    return items
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
  return json
    // Just型
    .replace(/"@@type":\s*"Just",\s*"value":\s*([^}]+)/g, (_, val) => `Just(${val.trim()})`)
    .replace(/\{\s*Just\(([^)]+)\)\s*\}/g, 'Just($1)')
    // Nothing
    .replace(/"@@Nothing"/g, 'Nothing')
    // Right型
    .replace(/"@@type":\s*"Right",\s*"value":\s*([^}]+)/g, (_, val) => `Right(${val.trim()})`)
    .replace(/\{\s*Right\(([^)]+)\)\s*\}/g, 'Right($1)')
    // Left型
    .replace(/"@@type":\s*"Left",\s*"value":\s*([^}]+)/g, (_, val) => `Left(${val.trim()})`)
    .replace(/\{\s*Left\(([^)]+)\)\s*\}/g, 'Left($1)')
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

const x = 42;

const y = "Hello";

const z = true;

show(x);

show(y);

show(z);

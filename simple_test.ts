// Generated TypeScript code from Seseragi

// Seseragi minimal runtime

type Maybe<T> = { tag: 'Just'; value: T } | { tag: 'Nothing' };

const Just = <T>(value: T): Maybe<T> => ({ tag: 'Just', value });
const Nothing: Maybe<never> = { tag: 'Nothing' };

const toString = (value: any): string => {
  // Maybeåã®ç¾ããè¡¨ç¤º
  if (value && typeof value === 'object' && value.tag === 'Just') {
    return `Just(${toString(value.value)})`
  }
  if (value && typeof value === 'object' && value.tag === 'Nothing') {
    return 'Nothing'
  }
  
  // Eitheråã®ç¾ããè¡¨ç¤º
  if (value && typeof value === 'object' && value.tag === 'Left') {
    return `Left(${toString(value.value)})`
  }
  if (value && typeof value === 'object' && value.tag === 'Right') {
    return `Right(${toString(value.value)})`
  }
  
  // Liståã®ç¾ããè¡¨ç¤º
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
  
  // Tupleåã®ç¾ããè¡¨ç¤º
  if (value && typeof value === 'object' && value.tag === 'Tuple') {
    return `(${value.elements.map(toString).join(', ')})`
  }
  
  // éåã®è¡¨ç¤º
  if (Array.isArray(value)) {
    return `[${value.map(toString).join(', ')}]`
  }
  
  // ããªããã£ãå
  if (typeof value === 'string') {
    return `"${value}"`
  }
  if (typeof value === 'number') {
    return String(value)
  }
  if (typeof value === 'boolean') {
    return value ? 'True' : 'False'
  }
  
  // æ®éã®ãªãã¸ã§ã¯ãï¼æ§é ä½ãªã©ï¼
  if (typeof value === 'object' && value !== null) {
    const pairs = []
    for (const key in value) {
      if (value.hasOwnProperty(key)) {
        pairs.push(`${key}: ${toString(value[key])}`)
      }
    }
    
    // æ§é ä½åãåå¾ï¼constructor.nameãä½¿ç¨ï¼
    const structName = value.constructor && value.constructor.name !== 'Object' 
      ? value.constructor.name 
      : ''
    
    // è¤æ°ãã£ã¼ã«ããããå ´åã¯ã¤ã³ãã³ãè¡¨ç¤º
    if (pairs.length > 2) {
      return `${structName} {\n  ${pairs.join(',\n  ')}\n}`
    } else {
      return `${structName} { ${pairs.join(', ')} }`
    }
  }
  
  return String(value)
};
// Seseragiåã®æ§é ãæ­£è¦å
function normalizeStructure(obj) {
  // ããªããã£ãåã®å¦ç
  if (typeof obj === 'boolean') {
    return { '@@type': 'Boolean', value: obj }
  }
  if (!obj || typeof obj !== 'object') return obj
  
  // Listå â ç¹å¥ãªãã¼ã«ã¼ä»ãéåã«å¤æ
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
  
  // Maybeå
  if (obj.tag === 'Just') {
    return { '@@type': 'Just', value: normalizeStructure(obj.value) }
  }
  if (obj.tag === 'Nothing') {
    return '@@Nothing'
  }
  
  // Eitherå
  if (obj.tag === 'Right') {
    return { '@@type': 'Right', value: normalizeStructure(obj.value) }
  }
  if (obj.tag === 'Left') {
    return { '@@type': 'Left', value: normalizeStructure(obj.value) }
  }
  
  // Tupleå
  if (obj.tag === 'Tuple') {
    return { '@@type': 'Tuple', value: obj.elements.map(normalizeStructure) }
  }
  
  // éå
  if (Array.isArray(obj)) {
    return obj.map(normalizeStructure)
  }
  
  // éå¸¸ã®ãªãã¸ã§ã¯ã
  const result = {}
  for (const key in obj) {
    if (obj.hasOwnProperty(key)) {
      result[key] = normalizeStructure(obj[key])
    }
  }
  return result
}

// JSONæå­åãSeseragiåã®ç¾ããè¡¨è¨ã«å¤æ
function beautifySeseragiTypes(json) {
  let result = json
  
  // Seseragiç¹æ®åã®å¤æ
  result = beautifySpecialTypes(result)
  
  // æ®éã®ãªãã¸ã§ã¯ãï¼æ§é ä½ãªã©ï¼ã®å¤æ
  result = beautifyStructObjects(result)
  
  return result
}

// Seseragiç¹æ®åï¼MaybeãEitherãListï¼ã®ç¾ããå¤æ
function beautifySpecialTypes(json) {
  return json
    // Listå
    .replace(/\{\s*"@@type":\s*"List",\s*"value":\s*\[\s*\]\s*\}/g, '`[]')
    .replace(/\{\s*"@@type":\s*"List",\s*"value":\s*\[([\s\S]*?)\]\s*\}/g, (match, content) => {
      const cleanContent = content.replace(/\s+/g, ' ').trim()
      return `\`[${cleanContent}]`
    })
    // Maybeå
    .replace(/"@@type":\s*"Just",\s*"value":\s*([^}]+)/g, (_, val) => `Just(${val.trim()})`)
    .replace(/\{\s*Just\(([^)]+)\)\s*\}/g, 'Just($1)')
    .replace(/"@@Nothing"/g, 'Nothing')
    // Eitherå
    .replace(/"@@type":\s*"Right",\s*"value":\s*([^}]+)/g, (_, val) => `Right(${val.trim()})`)
    .replace(/\{\s*Right\(([^)]+)\)\s*\}/g, 'Right($1)')
    .replace(/"@@type":\s*"Left",\s*"value":\s*([^}]+)/g, (_, val) => `Left(${val.trim()})`)
    .replace(/\{\s*Left\(([^)]+)\)\s*\}/g, 'Left($1)')
    // ã¿ãã«å
    .replace(/\{\s*"@@type":\s*"Tuple",\s*"value":\s*\[([\s\S]*?)\]\s*\}/g, (match, content) => {
      const cleanContent = content.replace(/\s+/g, ' ').trim()
      return `(${cleanContent})`
    })
    // ãã¼ã«å¤å
    .replace(/\{\s*"@@type":\s*"Boolean",\s*"value":\s*(true|false)\s*\}/g, (match, value) => {
      return value === 'true' ? 'True' : 'False'
    })
}

// æ®éã®ãªãã¸ã§ã¯ãï¼æ§é ä½ï¼ã®ç¾ããå¤æ
function beautifyStructObjects(json) {
  return json.replace(/\{([\s\S]*?)\}/g, (match, content) => {
    // æ¢ã«å¤ææ¸ã¿ã®Seseragiåã¯é¤å¤
    if (match.includes('Just(') || match.includes('Right(') || match.includes('Left(') || match.includes('`[')) {
      return match
    }
    
    // ãã£ã¼ã«ããè§£æ
    const fields = content.trim().split(',').filter(f => f.trim())
    const jsFields = fields.map(field => {
      const cleaned = field.trim().replace(/"(\w+)":/g, '$1:')
      return cleaned
    })
    
    // è¤æ°ãã£ã¼ã«ãã®å ´åã¯ã¤ã³ãã³ãè¡¨ç¤ºãä¿æãå°æ°ãã£ã¼ã«ãã¯1è¡
    if (jsFields.length > 2) {
      return `{\n  ${jsFields.join(',\n  ')}\n}`
    } else {
      return `{ ${jsFields.join(', ')} }`
    }
  })
}


// ç¾ãããã©ã¼ãããããé¢æ°
const prettyFormat = (value) => {
  // ããªããã£ãå
  if (typeof value === 'string') return `"${value}"`
  if (typeof value === 'number') return String(value)
  if (typeof value === 'boolean') return value ? 'True' : 'False'
  if (value === null) return 'null'
  if (value === undefined) return 'undefined'
  
  // Seseragiç¹æ®åã¨ãªãã¸ã§ã¯ãã®å ´å
  if (value && typeof value === 'object') {
    // Maybeå
    if (value.tag === 'Just') {
      return `Just(${prettyFormat(value.value)})`
    }
    if (value.tag === 'Nothing') {
      return 'Nothing'
    }
    
    // Eitherå
    if (value.tag === 'Left') {
      return `Left(${prettyFormat(value.value)})`
    }
    if (value.tag === 'Right') {
      return `Right(${prettyFormat(value.value)})`
    }
    
    // Listå
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
    
    // Tupleå
    if (value.tag === 'Tuple') {
      return `(${value.elements.map(prettyFormat).join(', ')})`
    }
    
    // éå
    if (Array.isArray(value)) {
      return `[${value.map(prettyFormat).join(', ')}]`
    }
    
    // æ§é ä½ã»æ®éã®ãªãã¸ã§ã¯ã
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

const arr = [1, 2, 3];

show(arr.length);

const x = ((0) >= 0 && (0) < (arr.tag === 'Tuple' ? arr.elements : arr).length ? { tag: 'Just', value: (arr.tag === 'Tuple' ? arr.elements : arr)[0] } : { tag: 'Nothing' });

show(x);

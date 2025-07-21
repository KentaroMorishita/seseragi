// Generated TypeScript code from Seseragi

// Seseragi runtime helpers

type Maybe<T> = { tag: 'Just'; value: T } | { tag: 'Nothing' };
type Either<L, R> = { tag: 'Left'; value: L } | { tag: 'Right'; value: R };
type List<T> = { tag: 'Empty' } | { tag: 'Cons'; head: T; tail: List<T> };

const curry = (fn: Function) => {
  return function curried(...args: any[]) {
    if (args.length >= fn.length) {
      return fn.apply(this, args);
    } else {
      return function(...args2: any[]) {
        return curried.apply(this, args.concat(args2));
      };
    }
  };
};

const pipe = <T, U>(value: T, fn: (arg: T) => U): U => fn(value);

const reversePipe = <T, U>(fn: (arg: T) => U, value: T): U => fn(value);

const map = <T, U>(fn: (value: T) => U, container: Maybe<T> | Either<any, T>): Maybe<U> | Either<any, U> => {
  if ('tag' in container) {
    if (container.tag === 'Just') return Just(fn(container.value));
    if (container.tag === 'Right') return Right(fn(container.value));
    if (container.tag === 'Nothing') return Nothing;
    if (container.tag === 'Left') return container;
  }
  return Nothing;
};

const applyWrapped = <T, U>(wrapped: Maybe<(value: T) => U> | Either<any, (value: T) => U>, container: Maybe<T> | Either<any, T>): Maybe<U> | Either<any, U> => {
  // Maybe types
  if (wrapped.tag === 'Nothing' || container.tag === 'Nothing') return Nothing;
  if (wrapped.tag === 'Just' && container.tag === 'Just') return Just(wrapped.value(container.value));
  // Either types
  if (wrapped.tag === 'Left') return wrapped;
  if (container.tag === 'Left') return container;
  if (wrapped.tag === 'Right' && container.tag === 'Right') return Right(wrapped.value(container.value));
  return Nothing;
};

const bind = <T, U>(container: Maybe<T> | Either<any, T>, fn: (value: T) => Maybe<U> | Either<any, U>): Maybe<U> | Either<any, U> => {
  if (container.tag === 'Just') return fn(container.value);
  if (container.tag === 'Right') return fn(container.value);
  if (container.tag === 'Nothing') return Nothing;
  if (container.tag === 'Left') return container;
  return Nothing;
};

const foldMonoid = <T>(arr: T[], empty: T, combine: (a: T, b: T) => T): T => {
  return arr.reduce(combine, empty);
};

// Array monadic functions
const mapArray = <T, U>(fa: T[], f: (a: T) => U): U[] => {
  return fa.map(f);
};

const applyArray = <T, U>(ff: ((a: T) => U)[], fa: T[]): U[] => {
  const result: U[] = [];
  for (const func of ff) {
    for (const value of fa) {
      result.push(func(value));
    }
  }
  return result;
};

const bindArray = <T, U>(ma: T[], f: (value: T) => U[]): U[] => {
  const result: U[] = [];
  for (const value of ma) {
    result.push(...f(value));
  }
  return result;
};

// List monadic functions
const mapList = <T, U>(fa: any, f: (a: T) => U): any => {
  if (fa.tag === 'Empty') return { tag: 'Empty' };
  return { tag: 'Cons', head: f(fa.head), tail: mapList(fa.tail, f) };
};

const applyList = <T, U>(ff: any, fa: any): any => {
  if (ff.tag === 'Empty') return { tag: 'Empty' };
  const mappedValues = mapList(fa, ff.head);
  const restApplied = applyList(ff.tail, fa);
  return concatList(mappedValues, restApplied);
};

const concatList = <T>(list1: any, list2: any): any => {
  if (list1.tag === 'Empty') return list2;
  return { tag: 'Cons', head: list1.head, tail: concatList(list1.tail, list2) };
};

const bindList = <T, U>(ma: any, f: (value: T) => any): any => {
  if (ma.tag === 'Empty') return { tag: 'Empty' };
  const headResult = f(ma.head);
  const tailResult = bindList(ma.tail, f);
  return concatList(headResult, tailResult);
};

// Maybe monadic functions
const mapMaybe = <T, U>(fa: Maybe<T>, f: (a: T) => U): Maybe<U> => {
  return fa.tag === 'Just' ? Just(f(fa.value)) : Nothing;
};

const applyMaybe = <T, U>(ff: Maybe<(a: T) => U>, fa: Maybe<T>): Maybe<U> => {
  return ff.tag === 'Just' && fa.tag === 'Just' ? Just(ff.value(fa.value)) : Nothing;
};

const bindMaybe = <T, U>(ma: Maybe<T>, f: (value: T) => Maybe<U>): Maybe<U> => {
  return ma.tag === 'Just' ? f(ma.value) : Nothing;
};

// Either monadic functions
const mapEither = <L, R, U>(ea: Either<L, R>, f: (value: R) => U): Either<L, U> => {
  return ea.tag === 'Right' ? Right(f(ea.value)) : ea;
};

const applyEither = <L, R, U>(ef: Either<L, (value: R) => U>, ea: Either<L, R>): Either<L, U> => {
  return ef.tag === 'Right' && ea.tag === 'Right' ? Right(ef.value(ea.value)) :
         ef.tag === 'Left' ? ef : ea;
};

const bindEither = <L, R, U>(ea: Either<L, R>, f: (value: R) => Either<L, U>): Either<L, U> => {
  return ea.tag === 'Right' ? f(ea.value) : ea;
};

const Just = <T>(value: T): Maybe<T> => ({ tag: 'Just', value });
const Nothing: Maybe<never> = { tag: 'Nothing' };

const Left = <L>(value: L): Either<L, never> => ({ tag: 'Left', value });
const Right = <R>(value: R): Either<never, R> => ({ tag: 'Right', value });

// Nullish coalescing helper functions
const fromMaybe = <T>(defaultValue: T, maybe: Maybe<T>): T => {
  return maybe.tag === 'Just' ? maybe.value : defaultValue;
};

const fromRight = <L, R>(defaultValue: R, either: Either<L, R>): R => {
  return either.tag === 'Right' ? either.value : defaultValue;
};

const fromLeft = <L, R>(defaultValue: L, either: Either<L, R>): L => {
  return either.tag === 'Left' ? either.value : defaultValue;
};

const Empty: List<never> = { tag: 'Empty' };
const Cons = <T>(head: T, tail: List<T>): List<T> => ({ tag: 'Cons', head, tail });

const headList = <T>(list: List<T>): Maybe<T> => list.tag === 'Cons' ? { tag: 'Just', value: list.head } : { tag: 'Nothing' };
const tailList = <T>(list: List<T>): List<T> => list.tag === 'Cons' ? list.tail : Empty;

const print = (value: any): void => {
  // Seseragi型の場合は美しく整形
  if (value && typeof value === 'object' && (
    value.tag === 'Just' || value.tag === 'Nothing' ||
    value.tag === 'Left' || value.tag === 'Right' ||
    value.tag === 'Cons' || value.tag === 'Empty'
  )) {
    console.log(toString(value))
  }
  // 通常のオブジェクトはそのまま
  else {
    console.log(value)
  }
};
const putStrLn = (value: string): void => console.log(value);
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
const toInt = (value: any): number => {
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
};
const toFloat = (value: any): number => {
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
};
const show = (value: any): void => {
  console.log(toString(value))
};

const arrayToList = curry(<T>(arr: T[]): List<T> => {
  let result: List<T> = Empty;
  for (let i = arr.length - 1; i >= 0; i--) {
    result = Cons(arr[i], result);
  }
  return result;
});

const listToArray = curry(<T>(list: List<T>): T[] => {
  const result: T[] = [];
  let current = list;
  while (current.tag === 'Cons') {
    result.push(current.head);
    current = current.tail;
  }
  return result;
});

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


type Node = (Array<string> | string);

type Props = { id: Maybe<string>, className: Maybe<string>, children: Node };

const add = curry(<A, B>(a: any, b: any): any => __dispatchOperator(a, "+", b));

const reduce = curry(<A, B>(f: (arg: any) => (arg: any) => any, acc: any, xs: Array<any>): any => (() => {
  const matchValue = xs;
  if ((matchValue.length >= 1)) {
    const x = matchValue[0];
    const rest = matchValue.slice(1);
    return reduce(f)(f(acc)(x))(rest);
  }  if (matchValue.length === 0) {
    return acc;
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})());

const getChildren = (props: Props): Node => (() => {
  const matchValue = props.children;
  if ((matchValue.length >= 1)) {
    const x = matchValue[0];
    const rest = matchValue.slice(1);
    return reduce(add)(x)(rest);
  }  if (matchValue.length === 0) {
    return "";
  }  if (true) {
    return props.children;
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

const genTag = curry((props: Props, tag: string): Node => (() => {
  const id = (() => {
  const matchValue = props.id;
  if (matchValue.tag === 'Just') {
    const id = matchValue.value;
    return `id=${id}`;
  }  if (matchValue.tag === 'Nothing') {
    return "";
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
  const className = (() => {
  const matchValue = props.className;
  if (matchValue.tag === 'Just') {
    const className = matchValue.value;
    return `className=${className}`;
  }  if (matchValue.tag === 'Nothing') {
    return "";
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
  const children = getChildren(props);
  return `<${tag} ${id} ${className}>${children}</${tag}>`;
})());

const div = (props: Props): Node => genTag(props)("div");

const p = (props: Props): Node => genTag(props)("p");

const app = (): Node => (() => {
  return div({ id: Just("app"), className: Just("container"), children: [div({ id: Just("header"), children: "Welcome to Seseragi!", className: Nothing }), div({ id: Just("content"), children: [p({ children: "This is a simple SSRG application.", id: Nothing, className: Nothing })], className: Nothing }), div({ id: Just("footer"), children: "© 2024 Seseragi Project", className: Nothing })] });
})();

show(app());

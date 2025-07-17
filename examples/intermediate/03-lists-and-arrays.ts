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

const Empty: List<never> = { tag: 'Empty' };
const Cons = <T>(head: T, tail: List<T>): List<T> => ({ tag: 'Cons', head, tail });

const headList = <T>(list: List<T>): Maybe<T> => list.tag === 'Cons' ? { tag: 'Just', value: list.head } : { tag: 'Nothing' };
const tailList = <T>(list: List<T>): List<T> => list.tag === 'Cons' ? list.tail : Empty;

const print = (value: any): void => {
  // Seseragiåã®å ´åã¯ç¾ããæ´å½¢
  if (value && typeof value === 'object' && (
    value.tag === 'Just' || value.tag === 'Nothing' ||
    value.tag === 'Left' || value.tag === 'Right' ||
    value.tag === 'Cons' || value.tag === 'Empty'
  )) {
    console.log(toString(value))
  }
  // éå¸¸ã®ãªãã¸ã§ã¯ãã¯ãã®ã¾ã¾
  else {
    console.log(value)
  }
};
const putStrLn = (value: string): void => console.log(value);
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
  // æ§é ä½ã®ãã£ã¼ã«ãã¢ã¯ã»ã¹ã®å ´åã¯ç´æ¥è¿ã
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


console.log("=== ListとArrayの基本 ===");

console.log("--- Array型 ---");

const numbers = [1, 2, 3, 4, 5];

const strings = ["hello", "world", "seseragi"];

const booleans = [true, false, true];

const empty = [];

show(numbers);

show(strings);

show(booleans);

show(empty);

show(numbers.length);

show(strings.length);

show(empty.length);

show(((0) >= 0 && (0) < (numbers.tag === 'Tuple' ? numbers.elements : numbers).length ? { tag: 'Just', value: (numbers.tag === 'Tuple' ? numbers.elements : numbers)[0] } : { tag: 'Nothing' }));

show(((2) >= 0 && (2) < (numbers.tag === 'Tuple' ? numbers.elements : numbers).length ? { tag: 'Just', value: (numbers.tag === 'Tuple' ? numbers.elements : numbers)[2] } : { tag: 'Nothing' }));

show(((1) >= 0 && (1) < (strings.tag === 'Tuple' ? strings.elements : strings).length ? { tag: 'Just', value: (strings.tag === 'Tuple' ? strings.elements : strings)[1] } : { tag: 'Nothing' }));

show(((10) >= 0 && (10) < (numbers.tag === 'Tuple' ? numbers.elements : numbers).length ? { tag: 'Just', value: (numbers.tag === 'Tuple' ? numbers.elements : numbers)[10] } : { tag: 'Nothing' }));

show(((5) >= 0 && (5) < (strings.tag === 'Tuple' ? strings.elements : strings).length ? { tag: 'Just', value: (strings.tag === 'Tuple' ? strings.elements : strings)[5] } : { tag: 'Nothing' }));

(() => {
  const matchValue = ((0) >= 0 && (0) < (numbers.tag === 'Tuple' ? numbers.elements : numbers).length ? { tag: 'Just', value: (numbers.tag === 'Tuple' ? numbers.elements : numbers)[0] } : { tag: 'Nothing' });
  if (matchValue.tag === 'Just') {
    const value = matchValue.value;
    return show(`First element: ${value}`);
  }  if (matchValue.tag === 'Nothing') {
    return show("No element found");
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

(() => {
  const matchValue = ((10) >= 0 && (10) < (strings.tag === 'Tuple' ? strings.elements : strings).length ? { tag: 'Just', value: (strings.tag === 'Tuple' ? strings.elements : strings)[10] } : { tag: 'Nothing' });
  if (matchValue.tag === 'Just') {
    const value = matchValue.value;
    return show(`Element at index 10: ${value}`);
  }  if (matchValue.tag === 'Nothing') {
    return show("Index out of bounds");
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

console.log("--- List型 ---");

const emptyList = Empty;

const emptyList_prime = Empty;

show(emptyList);

show(emptyList_prime);

const singletonList = Cons(42, Empty);

show(singletonList);

const singletonList_prime = Cons(42, Empty);

show(singletonList_prime);

const singletonList_prime_prime = Cons(42, Empty);

show(singletonList_prime_prime);

const list1 = Cons(1, Cons(2, Cons(3, Empty)));

const list2 = Cons(1, Cons(2, Cons(3, Empty)));

const list3 = Cons(1, Cons(2, Cons(3, Empty)));

show(list1);

show(list2);

show(list3);

const stringList = Cons("hello", Cons("world", Cons("seseragi", Empty)));

show(stringList);

const numbers_prime = Cons(10, Cons(20, Cons(30, Cons(40, Empty))));

show(headList(numbers_prime));

show(tailList(numbers_prime));

show(headList(numbers_prime));

show(tailList(numbers_prime));

show(headList(tailList(numbers_prime)));

show(tailList(tailList(numbers_prime)));

show(headList(tailList(tailList(numbers_prime))));

show(headList(tailList(tailList(tailList(numbers_prime)))));

const emptyList_prime_prime = Empty;

show(headList(emptyList_prime_prime));

show(tailList(emptyList_prime_prime));

console.log("--- Array↔List変換 ---");

const arr: Array<number> = [1, 2, 3, 4, 5];

const list: List<number> = arrayToList(arr);

show(arr);

show(list);

const backToArray: Array<number> = listToArray(list);

show(backToArray);

const emptyArr: Array<number> = [];

const emptyListFromArray: List<number> = arrayToList(emptyArr);

const emptyArrFromList: Array<number> = listToArray(emptyListFromArray);

show(emptyArr);

show(emptyListFromArray);

show(emptyArrFromList);

console.log("--- Array内包表記 ---");

const squares = [1, 2, 3, 4, 5].map(x => (x * x));

show(squares);

const range1 = Array.from({length: 5 - 1}, (_, i) => i + 1);

const range2 = Array.from({length: 5 - 1 + 1}, (_, i) => i + 1);

show(range1);

show(range2);

const squaresRange = Array.from({length: 5 - 1 + 1}, (_, i) => i + 1).map(x => (x * x));

show(squaresRange);

const evenSquares = Array.from({length: 6 - 1 + 1}, (_, i) => i + 1).filter(x => ((x % 2) === 0)).map(x => (x * x));

show(evenSquares);

const pairs = Array.from({length: 2 - 1 + 1}, (_, i) => i + 1).flatMap(x => Array.from({length: 4 - 3 + 1}, (_, i) => i + 3).map(y => [x, y])).map(tuple => {
          const [x, y] = tuple;
          return { tag: 'Tuple', elements: [x, y] };
        });

show(pairs);

console.log("--- List内包表記 ---");

const squares_prime = arrayToList([1, 2, 3, 4, 5].map(x => (x * x)));

show(squares_prime);

const squaresRange_prime = arrayToList(Array.from({length: 5 - 1 + 1}, (_, i) => i + 1).map(x => (x * x)));

show(squaresRange_prime);

const evenSquares_prime = arrayToList(Array.from({length: 6 - 1 + 1}, (_, i) => i + 1).filter(x => ((x % 2) === 0)).map(x => (x * x)));

show(evenSquares_prime);

const greetings = arrayToList(["Alice", "Bob", "Charlie"].map(name => `Hello, ${name}!`));

show(greetings);

const filtered = arrayToList(Array.from({length: 3 - 1 + 1}, (_, i) => i + 1).flatMap(x => Array.from({length: 6 - 4 + 1}, (_, i) => i + 4).map(y => [x, y])).filter(tuple => {
            const [x, y] = tuple;
            return ((x + y) > 6);
          }).map(tuple => {
          const [x, y] = tuple;
          return (x + y);
        }));

show(filtered);

console.log("--- ArrayとListの使い分け ---");

console.log("Array型は型安全なインデックスアクセス、List型は関数プログラミングに適しています");

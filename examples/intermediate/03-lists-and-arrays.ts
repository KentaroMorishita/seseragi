// Generated TypeScript code from Seseragi

import type {
  Maybe, Either, List, Signal
} from '@seseragi/runtime';

import {
  // 基本ユーティリティ
  pipe, reversePipe, map, applyWrapped, bind, foldMonoid,
  // Unit型
  Unit,
  // Maybe型
  Just, Nothing, mapMaybe, applyMaybe, bindMaybe, fromMaybe,
  // Either型
  Left, Right, mapEither, applyEither, bindEither, fromRight, fromLeft,
  // List型
  Empty, Cons, headList, tailList, mapList, applyList, concatList, bindList,
  // Array型
  mapArray, applyArray, bindArray, arrayToList, listToArray,
  // Task型
  Task, resolve, ssrgRun, ssrgTryRun, mapTask, applyTask, bindTask,
  // Signal型
  createSignal, setSignal, subscribeSignal, unsubscribeSignal, detachSignal,
  mapSignal, applySignal, bindSignal, ssrgSignalSubscribe, ssrgSignalUnsubscribe, ssrgSignalDetach,
  // 組み込み関数
  ssrgPrint, ssrgPutStrLn, ssrgToString, ssrgToInt, ssrgToFloat, ssrgShow,
  // 型システム
  __typeRegistry, __variableTypes, __variableAliases,
  ssrgTypeOf, ssrgTypeOfWithAliases, ssrgIsType
} from '@seseragi/runtime';


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
    case '+': {
      // 型安全な加算：両方が数値の場合のみ数値演算、それ以外は文字列連結
      if (typeof left === 'number' && typeof right === 'number') return left + right;
      if (typeof left === 'string' || typeof right === 'string') return String(left) + String(right);
      return left + right;
    }
    case '-': return Number(left) - Number(right);
    case '*': return Number(left) * Number(right);
    case '/': return Number(left) / Number(right);
    case '%': return Number(left) % Number(right);
    case '**': return Number(left) ** Number(right);
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
  "numbers": "Array<Int>",
  "strings": "Array<String>",
  "booleans": "Array<Bool>",
  "empty": "Array<unknown>",
  "emptyList": "List<unknown>",
  "emptyList'": "List<unknown>",
  "singletonList": "List<Int>",
  "singletonList'": "List<Int>",
  "singletonList''": "List<Int>",
  "list1": "List<Int>",
  "list2": "List<Int>",
  "list3": "List<Int>",
  "stringList": "List<String>",
  "numbers'": "List<Int>",
  "emptyList''": "List<unknown>",
  "arr": "Array<Int>",
  "list": "List<Int>",
  "backToArray": "Array<Int>",
  "emptyArr": "Array<Int>",
  "emptyListFromArray": "List<Int>",
  "emptyArrFromList": "Array<Int>",
  "squares": "Array<Int>",
  "range1": "Array<Int>",
  "range2": "Array<Int>",
  "squaresRange": "Array<Int>",
  "evenSquares": "Array<Int>",
  "pairs": "Array<(Int, Int)>",
  "squares'": "List<Int>",
  "squaresRange'": "List<Int>",
  "evenSquares'": "List<Int>",
  "greetings": "List<String>",
  "filtered": "List<Int>"
});

ssrgPrint("=== ListとArrayの基本 ===");

ssrgPrint("--- Array型 ---");

const numbers = [1, 2, 3, 4, 5];

const strings = ["hello", "world", "seseragi"];

const booleans = [true, false, true];

const empty = [];

ssrgShow(numbers);

ssrgShow(strings);

ssrgShow(booleans);

ssrgShow(empty);

ssrgShow(numbers.length);

ssrgShow(strings.length);

ssrgShow(empty.length);

ssrgShow(((0) >= 0 && (0) < (numbers.tag === 'Tuple' ? numbers.elements : numbers).length ? { tag: 'Just', value: (numbers.tag === 'Tuple' ? numbers.elements : numbers)[0] } : { tag: 'Nothing' }));

ssrgShow(((2) >= 0 && (2) < (numbers.tag === 'Tuple' ? numbers.elements : numbers).length ? { tag: 'Just', value: (numbers.tag === 'Tuple' ? numbers.elements : numbers)[2] } : { tag: 'Nothing' }));

ssrgShow(((1) >= 0 && (1) < (strings.tag === 'Tuple' ? strings.elements : strings).length ? { tag: 'Just', value: (strings.tag === 'Tuple' ? strings.elements : strings)[1] } : { tag: 'Nothing' }));

ssrgShow(((10) >= 0 && (10) < (numbers.tag === 'Tuple' ? numbers.elements : numbers).length ? { tag: 'Just', value: (numbers.tag === 'Tuple' ? numbers.elements : numbers)[10] } : { tag: 'Nothing' }));

ssrgShow(((5) >= 0 && (5) < (strings.tag === 'Tuple' ? strings.elements : strings).length ? { tag: 'Just', value: (strings.tag === 'Tuple' ? strings.elements : strings)[5] } : { tag: 'Nothing' }));

(() => {
  const matchValue = ((0) >= 0 && (0) < (numbers.tag === 'Tuple' ? numbers.elements : numbers).length ? { tag: 'Just', value: (numbers.tag === 'Tuple' ? numbers.elements : numbers)[0] } : { tag: 'Nothing' });
  if (matchValue.tag === 'Just') {
    const value = matchValue.value;
    return ssrgShow(`First element: ${value}`);
  }  if (matchValue.tag === 'Nothing') {
    return ssrgShow("No element found");
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

(() => {
  const matchValue = ((10) >= 0 && (10) < (strings.tag === 'Tuple' ? strings.elements : strings).length ? { tag: 'Just', value: (strings.tag === 'Tuple' ? strings.elements : strings)[10] } : { tag: 'Nothing' });
  if (matchValue.tag === 'Just') {
    const value = matchValue.value;
    return ssrgShow(`Element at index 10: ${value}`);
  }  if (matchValue.tag === 'Nothing') {
    return ssrgShow("Index out of bounds");
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

ssrgPrint("--- List型 ---");

const emptyList = Empty;

const emptyList_prime = Empty;

ssrgShow(emptyList);

ssrgShow(emptyList_prime);

const singletonList = Cons(42, Empty);

ssrgShow(singletonList);

const singletonList_prime = Cons(42, Empty);

ssrgShow(singletonList_prime);

const singletonList_prime_prime = Cons(42, Empty);

ssrgShow(singletonList_prime_prime);

const list1 = Cons(1, Cons(2, Cons(3, Empty)));

const list2 = Cons(1, Cons(2, Cons(3, Empty)));

const list3 = Cons(1, Cons(2, Cons(3, Empty)));

ssrgShow(list1);

ssrgShow(list2);

ssrgShow(list3);

const stringList = Cons("hello", Cons("world", Cons("seseragi", Empty)));

ssrgShow(stringList);

const numbers_prime = Cons(10, Cons(20, Cons(30, Cons(40, Empty))));

ssrgShow(headList(numbers_prime));

ssrgShow(tailList(numbers_prime));

ssrgShow(headList(numbers_prime));

ssrgShow(tailList(numbers_prime));

ssrgShow(headList(tailList(numbers_prime)));

ssrgShow(tailList(tailList(numbers_prime)));

ssrgShow(headList(tailList(tailList(numbers_prime))));

ssrgShow(headList(tailList(tailList(tailList(numbers_prime)))));

const emptyList_prime_prime = Empty;

ssrgShow(headList(emptyList_prime_prime));

ssrgShow(tailList(emptyList_prime_prime));

ssrgPrint("--- Array↔List変換 ---");

const arr: Array<number> = [1, 2, 3, 4, 5];

const list: List<number> = arrayToList(arr);

ssrgShow(arr);

ssrgShow(list);

const backToArray: Array<number> = listToArray(list);

ssrgShow(backToArray);

const emptyArr: Array<number> = [];

const emptyListFromArray: List<number> = arrayToList(emptyArr);

const emptyArrFromList: Array<number> = listToArray(emptyListFromArray);

ssrgShow(emptyArr);

ssrgShow(emptyListFromArray);

ssrgShow(emptyArrFromList);

ssrgPrint("--- Array内包表記 ---");

const squares = [1, 2, 3, 4, 5].map(x => (x * x));

ssrgShow(squares);

const range1 = Array.from({length: 5 - 1}, (_, i) => i + 1);

const range2 = Array.from({length: 5 - 1 + 1}, (_, i) => i + 1);

ssrgShow(range1);

ssrgShow(range2);

const squaresRange = Array.from({length: 5 - 1 + 1}, (_, i) => i + 1).map(x => (x * x));

ssrgShow(squaresRange);

const evenSquares = Array.from({length: 6 - 1 + 1}, (_, i) => i + 1).filter(x => ((x % 2) === 0)).map(x => (x * x));

ssrgShow(evenSquares);

const pairs = Array.from({length: 2 - 1 + 1}, (_, i) => i + 1).flatMap(x => Array.from({length: 4 - 3 + 1}, (_, i) => i + 3).map(y => [x, y])).map(tuple => {
          const [x, y] = tuple;
          return { tag: 'Tuple', elements: [x, y] };
        });

ssrgShow(pairs);

ssrgPrint("--- List内包表記 ---");

const squares_prime = arrayToList([1, 2, 3, 4, 5].map(x => (x * x)));

ssrgShow(squares_prime);

const squaresRange_prime = arrayToList(Array.from({length: 5 - 1 + 1}, (_, i) => i + 1).map(x => (x * x)));

ssrgShow(squaresRange_prime);

const evenSquares_prime = arrayToList(Array.from({length: 6 - 1 + 1}, (_, i) => i + 1).filter(x => ((x % 2) === 0)).map(x => (x * x)));

ssrgShow(evenSquares_prime);

const greetings = arrayToList(["Alice", "Bob", "Charlie"].map(name => `Hello, ${name}!`));

ssrgShow(greetings);

const filtered = arrayToList(Array.from({length: 3 - 1 + 1}, (_, i) => i + 1).flatMap(x => Array.from({length: 6 - 4 + 1}, (_, i) => i + 4).map(y => [x, y])).filter(tuple => {
            const [x, y] = tuple;
            return ((x + y) > 6);
          }).map(tuple => {
          const [x, y] = tuple;
          return (x + y);
        }));

ssrgShow(filtered);

ssrgPrint("--- ArrayとListの使い分け ---");

ssrgPrint("Array型は型安全なインデックスアクセス、List型は関数プログラミングに適しています");

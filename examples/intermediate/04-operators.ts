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
  "maybeValue": "Maybe<Int>",
  "nothingValue": "Maybe<unknown>",
  "doubled": "Maybe<Int>",
  "doubled'": "Maybe<Int>",
  "rightValue": "Either<unknown, Int>",
  "leftValue": "Either<String, unknown>",
  "incremented": "Either<unknown, Int>",
  "incremented'": "Either<String, Int>",
  "value1": "Maybe<Int>",
  "value2": "Maybe<Int>",
  "value3": "Maybe<unknown>",
  "sum": "Maybe<Int>",
  "sum'": "Maybe<Int>",
  "right1": "Either<unknown, Int>",
  "right2": "Either<unknown, Int>",
  "left1": "Either<String, unknown>",
  "product": "Either<unknown, Int>",
  "product'": "Either<unknown, Int>",
  "result1": "Maybe<Int>",
  "result2": "Maybe<Int>",
  "chain1": "Maybe<Int>",
  "chain2": "Maybe<Int>",
  "valid1": "Either<unknown, Int>",
  "valid2": "Either<unknown, Int>",
  "valid3": "Either<unknown, Int>",
  "arr": "Array<Int>",
  "mapped": "Array<Int>",
  "lst": "List<Int>",
  "transformed": "List<Int>",
  "flattened": "Array<Int>",
  "listFlattened": "List<Int>",
  "x'": "Maybe<Int>",
  "y'": "Maybe<Int>",
  "result'": "Maybe<Int>",
  "value": "Maybe<Int>",
  "value'": "Maybe<String>",
  "nameValue": "Maybe<String>",
  "domainValue": "Maybe<String>",
  "email": "Maybe<String>",
  "result": "Maybe<String>"
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

const doubled = mapMaybe(maybeValue, double);

const doubled_prime = mapMaybe(nothingValue, double);

ssrgShow(doubled);

ssrgShow(doubled_prime);

const rightValue = Right(10);

const leftValue = Left("error");

const incremented = mapEither(rightValue, increment);

const incremented_prime = mapEither(leftValue, increment);

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

const sum = applyMaybe(applyMaybe(Just(add), value1), value2);

const sum_prime = applyMaybe(applyMaybe(Just(add), value1), value3);

ssrgShow(sum);

ssrgShow(sum_prime);

const right1 = Right(20);

const right2 = Right(3);

const left1 = Left("first error");

const product = applyEither(applyEither(Right(multiply), right1), right2);

const product_prime = applyEither(applyEither(Right(multiply), left1), right2);

ssrgShow(product);

ssrgShow(product_prime);

ssrgPrint("--- モナド演算子 >>= ---");

function doubleIfEven(x: number): Maybe<number> {
  return (((x % 2) === 0) ? Just((x * 2)) : Nothing);
}

function addTen(x: number): Maybe<number> {
  return ((x > 0) ? Just((x + 10)) : Nothing);
}

const result1 = bindMaybe(Just(4), doubleIfEven);

const result2 = bindMaybe(Just(3), doubleIfEven);

ssrgShow(result1);

ssrgShow(result2);

const chain1 = bindMaybe(bindMaybe(Just(2), doubleIfEven), addTen);

const chain2 = bindMaybe(bindMaybe(Just(3), doubleIfEven), addTen);

ssrgShow(chain1);

ssrgShow(chain2);

function validatePositive(x: number): Either<string, number> {
  return ((x > 0) ? Right(x) : Left("値は正数である必要があります"));
}

function validateEven(x: number): Either<string, number> {
  return (((x % 2) === 0) ? Right(x) : Left("値は偶数である必要があります"));
}

const valid1 = bindEither(bindEither(Right(8), validatePositive), validateEven);

const valid2 = bindEither(bindEither(Right(7), validatePositive), validateEven);

const valid3 = bindEither(bindEither(Right(-2), validatePositive), validateEven);

ssrgShow(valid1);

ssrgShow(valid2);

ssrgShow(valid3);

const arr = [1, 2, 3];

const mapped = mapArray(arr, (x: any) => (x * 2));

ssrgShow(mapped);

const lst = Cons(1, Cons(2, Cons(3, Empty)));

const transformed = mapList(lst, (x: any) => (x * 3));

ssrgShow(transformed);

function repeat(x: number): Array<number> {
  return [x, x];
}

const flattened = bindArray(arr, repeat);

ssrgShow(flattened);

function listRepeat(x: number): List<number> {
  return Cons(x, Cons(x, Empty));
}

const listFlattened = bindList(lst, listRepeat);

ssrgShow(listFlattened);

ssrgPrint("--- 実用的な例 ---");

const x_prime = Just(100);

const y_prime = Just(25);

function subtract(x: number): (arg: number) => number {
  return function(y: number) {
      return (x - y);
    };
}

const result_prime = applyMaybe(applyMaybe(Just(subtract), x_prime), y_prime);

ssrgShow(result_prime);

const value = Just(100);

const value_prime = mapMaybe(value, toString);

ssrgShow(value_prime);

function createEmail(name: string): (arg: string) => string {
  return function(domain: string) {
      return `${name}@${domain}`;
    };
}

const nameValue = Just("user");

const domainValue = Just("example.com");

const email = applyMaybe(applyMaybe(Just(createEmail), nameValue), domainValue);

ssrgShow(email);

function parsePort(input: string): Maybe<number> {
  return ((input === "80") ? Just(80) : ((input === "443") ? Just(443) : Nothing));
}

function validatePort(port: number): Maybe<string> {
  return ((port === 80) ? Just("HTTP") : ((port === 443) ? Just("HTTPS") : Nothing));
}

const result = bindMaybe(bindMaybe(Just("80"), parsePort), validatePort);

ssrgShow(result);

ssrgPrint("--- 演算子の利点 ---");

ssrgPrint("モナド演算子により、安全で表現力豊かなコードが書けます");

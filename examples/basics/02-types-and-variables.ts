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
  "userName": "String",
  "userAge": "Int",
  "x": "Int",
  "x'": "Int",
  "result'": "Int",
  "user_name": "String",
  "max_count": "Int",
  "number": "Int",
  "message": "String",
  "flag": "Bool",
  "pi": "Float",
  "sum": "Int",
  "difference": "Int",
  "product": "Int",
  "quotient": "Int",
  "isEqual": "Bool",
  "isNotEqual": "Bool",
  "isGreater": "Bool",
  "isLess": "Bool",
  "name": "String",
  "greeting": "String"
});

ssrgPrint("=== 基本型と変数 ===");

ssrgPrint("--- 命名規則 ---");

const userName = "Alice";

const userAge = 25;

const x = 10;

const x_prime = 20;

const result_prime = (x + x_prime);

const user_name = "Bob";

const max_count = 100;

ssrgShow(userName);

ssrgShow(x_prime);

ssrgShow(result_prime);

ssrgShow(user_name);

ssrgPrint("--- 基本型 ---");

const number = 42;

ssrgShow(number);

const message = "Hello, Seseragi!";

ssrgShow(message);

const flag = true;

ssrgShow(flag);

const pi = 3.14;

ssrgShow(pi);

ssrgPrint("--- 算術演算 ---");

const sum = (10 + 5);

const difference = (10 - 5);

const product = (6 * 7);

const quotient = Math.trunc(20 / 4);

ssrgShow(sum);

ssrgShow(difference);

ssrgShow(product);

ssrgShow(quotient);

ssrgPrint("--- 比較演算 ---");

const isEqual = (5 === 5);

const isNotEqual = (5 !== 3);

const isGreater = (7 > 4);

const isLess = (3 < 7);

ssrgShow(isEqual);

ssrgShow(isNotEqual);

ssrgShow(isGreater);

ssrgShow(isLess);

ssrgPrint("--- 文字列操作 ---");

const name = "Alice";

const greeting = `Hello, ${name}!`;

ssrgShow(greeting);

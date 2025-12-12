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
  "x": "Int",
  "result": "String",
  "isTrue": "Bool",
  "message": "String",
  "isFalse": "Bool",
  "message'": "String",
  "a": "Int",
  "b": "Int",
  "p": "Bool",
  "q": "Bool"
});

ssrgPrint("=== 条件分岐 ===");

ssrgPrint("--- 基本的な条件分岐 ---");

const x = 10;

const result = ((x > 5) ? "大きい" : "小さい");

ssrgShow(result);

const isTrue = true;

const message = (isTrue ? "真です" : "偽です");

ssrgShow(message);

const isFalse = false;

const message_prime = (isFalse ? "真です" : "偽です");

ssrgShow(message_prime);

ssrgPrint("--- 比較演算子 ---");

const a = 5;

const b = 3;

ssrgShow((a === b));

ssrgShow((a !== b));

ssrgShow((a > b));

ssrgShow((a < b));

ssrgShow((a >= b));

ssrgShow((a <= b));

ssrgPrint("--- ブール演算子 ---");

const p = true;

const q = false;

ssrgShow((p && q));

ssrgShow((p || q));

ssrgShow((!p));

ssrgPrint("--- 複合条件の書き方比較 ---");

function classify(age: number): string {
  return ((age < 13) ? "子供" : ((age < 20) ? "ティーンエイジャー" : ((age < 65) ? "大人" : "高齢者")));
}

function classify_prime(age: number): string {
  return ((age < 13) ? "子供" : ((age < 20) ? "ティーンエイジャー" : ((age < 65) ? "大人" : "高齢者")));
}

ssrgShow(classify(16));

ssrgShow(classify_prime(16));

ssrgShow(classify(10));

ssrgShow(classify_prime(30));

ssrgPrint("--- 実用的な例 ---");

function isEven(x: number): boolean {
  return ((x % 2) === 0);
}

function isOdd(x: number): boolean {
  return ((x % 2) !== 0);
}

ssrgShow(isEven(4));

ssrgShow(isOdd(4));

function grade(score: number): string {
  return ((score >= 90) ? "A" : ((score >= 80) ? "B" : ((score >= 70) ? "C" : ((score >= 60) ? "D" : "F"))));
}

ssrgShow(grade(95));

ssrgShow(grade(75));

ssrgShow(grade(45));

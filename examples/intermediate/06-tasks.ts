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
  "t1": "Task<Int>",
  "t1Logged": "Task<Int>",
  "t1DoubledLogged": "Task<Int>",
  "a": "Task<Int>",
  "b": "Task<Int>",
  "sumTask": "Task<Int>",
  "sumLogged": "Task<Int>",
  "chained": "Task<Int>",
  "chainedLogged": "Task<Int>",
  "failed": "Task<Int>",
  "ignored": "Promise<Either<String, Int>>"
});

ssrgPrint("=== Task の基本 ===");

const t1: Task<number> = Task(() => Promise.resolve(100));

function tapReturn(x: number): number {
  return ((_unit: void) => x)(ssrgPrint(`t1=${x}`));
}

const t1Logged: Task<number> = mapTask(tapReturn, t1);

ssrgRun(t1Logged);

function double(x: number): number {
  return (x * 2);
}

function log(x: number): number {
  return ((_unit: void) => x)(ssrgPrint(`double=${x}`));
}

const t1DoubledLogged: Task<number> = mapTask(log, mapTask(double, t1));

ssrgRun(t1DoubledLogged);

function add(x: number): (arg: number) => number {
  return function(y: number) {
      return (x + y);
    };
}

const a: Task<number> = Task(() => Promise.resolve(3));

const b: Task<number> = Task(() => Promise.resolve(7));

const sumTask: Task<number> = applyTask(mapTask(add, a), b);

const sumLogged: Task<number> = mapTask((s: any) => ((_unit: void) => s)(ssrgPrint(`sum=${s}`)), sumTask);

ssrgRun(sumLogged);

function times5(x: number): Task<number> {
  return Task(() => Promise.resolve((x * 5)));
}

const chained: Task<number> = bindTask(bindTask(t1, times5), times5);

const chainedLogged: Task<number> = mapTask((v: any) => ((_unit: void) => v)(ssrgPrint(`chained=${v}`)), chained);

ssrgRun(chainedLogged);

const failed: Task<number> = Task(() => Promise.reject<number>("boom"));

const ignored = ssrgTryRun(failed);

ssrgPrint("Taskデモ 終了");

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
  "s": "Signal<Int>",
  "key": "String",
  "doubled": "Signal<Int>",
  "key2": "String",
  "sumSig": "Signal<Int>",
  "key3": "String",
  "switched": "Signal<Int>",
  "key4": "String",
  "now": "Int"
});

ssrgPrint("=== FRP: Signalの基本 ===");

const s: Signal<number> = createSignal(0);

const key = ssrgSignalSubscribe(s, (v: any) => ssrgPrint(`s=${v}`));

s.setValue(1);

s.setValue(2);

function double(x: number): number {
  return (x * 2);
}

const doubled: Signal<number> = mapSignal(s, double);

const key2 = ssrgSignalSubscribe(doubled, (v: any) => ssrgPrint(`doubled=${v}`));

s.setValue(3);

function add(x: number): (arg: number) => number {
  return function(y: number) {
      return (x + y);
    };
}

const sumSig: Signal<number> = applySignal(mapSignal(s, add), doubled);

const key3 = ssrgSignalSubscribe(sumSig, (v: any) => ssrgPrint(`sumSig=${v}`));

s.setValue(4);

function toInner(n: number): Signal<number> {
  return createSignal((n * 10));
}

const switched: Signal<number> = bindSignal(s, toInner);

const key4 = ssrgSignalSubscribe(switched, (v: any) => ssrgPrint(`switched=${v}`));

s.setValue(5);

const now: number = (s.getValue());

ssrgPrint(`now=${now}`);

ssrgSignalUnsubscribe(key);

ssrgSignalUnsubscribe(key2);

ssrgSignalUnsubscribe(key3);

ssrgSignalUnsubscribe(key4);

ssrgSignalDetach(s);

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


// Initialize dispatch tables immediately
(() => {
  // Initialize method dispatch table
  __structMethods = {
    "Point": {
      "distanceSquaredTo": __ssrg_Point_f4pl4mu_distanceSquaredTo
    },
  };

  // Initialize operator dispatch table
  __structOperators = {
    "Point": {

    },
  };

})();

// 変数型情報テーブルの初期化
Object.assign(__variableTypes, {
  "number": "Int",
  "message": "String",
  "myList": "List<Int>",
  "point1": "Point { x: Int, y: Int }",
  "point2": "Point { x: Int, y: Int }",
  "point3": "Point { x: Int, y: Int }",
  "dist1": "Int",
  "dist2": "Int"
});

class Point {
  x: number;
  y: number;

  constructor(fields: { x: number, y: number }) {
    this.x = fields.x;
    this.y = fields.y;
  }
}

// Point implementation
function __ssrg_Point_f4pl4mu_distanceSquaredTo(self: A, other: B): number {
  return (() => {
  const dx = (other.x - self.x);
  const dy = (other.y - self.y);
  return ((dx * dx) + (dy * dy));
})();
}

ssrgShow("=== Seseragi プレイグラウンド ===");

const number = 42;

const message = "Hello, Seseragi!";

ssrgShow(number);

ssrgShow(message);

function greet(name: string): string {
  return `Hello, ${name}!`;
}

ssrgShow(greet("World"));

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

ssrgShow(add(5)(3));

ssrgShow(multiply(4)(7));

const myList = arrayToList(Array.from({length: 10 - 1 + 1}, (_, i) => i + 1).filter(x => ((x % 2) === 0)).map(x => x));

ssrgShow(myList);

const point1 = new Point({ x: 0, y: 0 });

const point2 = new Point({ x: 3, y: 4 });

const point3 = new Point({ x: 1, y: 1 });

ssrgShow(point1);

ssrgShow(point2);

ssrgShow(point3);

ssrgShow(__dispatchMethod(point1, "distanceSquaredTo", point2));

ssrgShow(__dispatchMethod(point1, "distanceSquaredTo", point3));

const dist1 = __dispatchMethod(point1, "distanceSquaredTo", point2);

const dist2 = __dispatchMethod(point1, "distanceSquaredTo", point3);

ssrgShow((dist1 > dist2));

ssrgShow(`point1から近い順: point3 (距離²=${dist2}) < point2 (距離²=${dist1})`);

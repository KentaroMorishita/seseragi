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
  "add5": "function",
  "triple": "function"
});

ssrgPrint("=== 関数 ===");

ssrgPrint("--- 関数の命名規則 ---");

function sayHello(): string {
  return "Hello!";
}

function getUserName(): string {
  return "User";
}

function calculate(x: number): number {
  return (x * 2);
}

function calculate_prime(x: number): number {
  return (x * 3);
}

ssrgShow(sayHello());

ssrgShow(calculate(5));

ssrgShow(calculate_prime(5));

ssrgPrint("--- 関数定義の基本構文 ---");

function hello(): string {
  return "Hello world";
}

ssrgShow(hello());

ssrgShow(hello);

function square(x: number): number {
  return (x * x);
}

ssrgShow(square(5));

function greet(name: string): string {
  return `Hello, ${name}!`;
}

ssrgShow(greet("Alice"));

ssrgPrint("--- 型の書き方 ---");

function getNumber(): number {
  return 42;
}

function getMessage(): string {
  return "Hello";
}

function getFlag(): boolean {
  return true;
}

function getPi(): number {
  return 3.14;
}

ssrgShow(getNumber());

ssrgShow(getMessage());

ssrgShow(getFlag());

ssrgShow(getPi());

function concat(first: string): (arg: string) => string {
  return function(second: string) {
      return (first + second);
    };
}

ssrgShow(concat("Hello")(" World"));

ssrgPrint("--- ブロック構文 ---");

function blockExample(x: number): number {
  return (() => {
  const doubled = (x * 2);
  const added = (doubled + 10);
  const result = (added * 3);
  return result;
})();
}

ssrgShow(blockExample(5));

function circleArea(radius: number): number {
  return (() => {
  const pi = 3.14159;
  const radiusSquared = (radius * radius);
  return (pi * radiusSquared);
})();
}

ssrgShow(circleArea(5));

function areaCalc(width: number): (arg: number) => number {
  return function(height: number) {
      return (() => {
  const area = (width * height);
  const message = `面積は ${area} です`;
  ssrgPrint(message);
  return area;
})();
    };
}

ssrgShow(areaCalc(5)(8));

ssrgPrint("--- カリー化された関数 ---");

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

const add5 = add(5);

const triple = multiply(3);

ssrgShow(add5(10));

ssrgShow(triple(7));

ssrgShow(add(3)(4));

ssrgShow(multiply(6)(9));

ssrgPrint("--- 高階関数 ---");

function applyFunc(func: (arg: number) => number): (arg: number) => number {
  return function(value: number) {
      return func(value);
    };
}

function quad(x: number): number {
  return (x * 4);
}

function increment(x: number): number {
  return (x + 1);
}

function addTen(x: number): number {
  return (x + 10);
}

ssrgShow(applyFunc(quad)(5));

ssrgShow(applyFunc(increment)(20));

ssrgShow(applyFunc(addTen)(5));

ssrgPrint("--- 実用的な関数例 ---");

function power(base: number): (arg: number) => number {
  return function(exp: number) {
      return ((exp === 0) ? 1 : (base * power(base)((exp - 1))));
    };
}

function factorial(n: number): number {
  return ((n <= 1) ? 1 : (n * factorial((n - 1))));
}

function repeatStr(str: string): (arg: number) => string {
  return function(times: number) {
      return ((times <= 0) ? "" : `${str}${repeatStr(str)((times - 1))}`);
    };
}

ssrgShow(power(2)(3));

ssrgShow(factorial(5));

ssrgShow(repeatStr("Hi ")(3));

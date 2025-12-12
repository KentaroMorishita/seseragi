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


// 変数型情報テーブルの初期化
Object.assign(__variableTypes, {
  "red": "Color",
  "green": "Color",
  "blue": "Color",
  "colorName": "String",
  "circle": "Shape",
  "rectangle": "Shape",
  "area": "Float",
  "just42": "Maybe<Int>",
  "nothing": "Maybe<unknown>",
  "maybeValue": "unknown",
  "right100": "Either<unknown, Int>",
  "leftError": "Either<String, unknown>",
  "eitherValue": "unknown",
  "myList": "List<Int>",
  "emptyList": "List<unknown>",
  "listHead": "unknown",
  "success": "Result",
  "zero": "Result",
  "failure": "Result"
});

ssrgPrint("=== パターンマッチング ===");

ssrgPrint("--- リテラルパターン ---");

function describeNumber(n: number): string {
  return (() => {
  const matchValue = n;
  if (matchValue === 0) {
    return "ゼロ";
  }  if (matchValue === 1) {
    return "一";
  }  if (matchValue === 42) {
    return "答え";
  }  if (true) {
    return "その他の数";
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

ssrgShow(describeNumber(0));

ssrgShow(describeNumber(42));

ssrgShow(describeNumber(99));

function greetByLanguage(lang: string): string {
  return (() => {
  const matchValue = lang;
  if (matchValue === "jp") {
    return "こんにちは";
  }  if (matchValue === "en") {
    return "Hello";
  }  if (matchValue === "fr") {
    return "Bonjour";
  }  if (true) {
    return "Unknown language";
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

ssrgShow(greetByLanguage("jp"));

ssrgShow(greetByLanguage("en"));

ssrgShow(greetByLanguage("de"));

ssrgPrint("--- タプルパターン ---");

function describePoint(point: [number, number]): string {
  return (() => {
  const matchValue = point;
  if (matchValue.elements[0] === 0 && matchValue.elements[1] === 0) {
    return "原点";
  }  if (matchValue.elements[0] === 0 && true) {
    return "Y軸上";
  }  if (true && matchValue.elements[1] === 0) {
    return "X軸上";
  }  if (true && true) {
    const x = matchValue.elements[0];
    const y = matchValue.elements[1];
    return `点(${x}, ${y})`;
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

ssrgShow(describePoint({ tag: 'Tuple', elements: [0, 0] }));

ssrgShow(describePoint({ tag: 'Tuple', elements: [0, 5] }));

ssrgShow(describePoint({ tag: 'Tuple', elements: [3, 0] }));

ssrgShow(describePoint({ tag: 'Tuple', elements: [2, 3] }));

function analyzeTriple(triple: [number, number, number]): string {
  return (() => {
  const matchValue = triple;
  if (matchValue.elements[0] === 0 && matchValue.elements[1] === 0 && matchValue.elements[2] === 0) {
    return "すべてゼロ";
  }  if (true && true && true) {
    const x = matchValue.elements[0];
    const y = matchValue.elements[1];
    const z = matchValue.elements[2];
    if (((x === y) && (y === z))) {
      return `すべて同じ値: ${x}`;
    }
  }  if (true && true && true) {
    const x = matchValue.elements[0];
    const y = matchValue.elements[1];
    const z = matchValue.elements[2];
    return `異なる値: ${x}, ${y}, ${z}`;
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

ssrgShow(analyzeTriple({ tag: 'Tuple', elements: [0, 0, 0] }));

ssrgShow(analyzeTriple({ tag: 'Tuple', elements: [5, 5, 5] }));

ssrgShow(analyzeTriple({ tag: 'Tuple', elements: [1, 2, 3] }));

ssrgPrint("--- 基本的なADT ---");

type Color = { type: 'Red' } | { type: 'Green' } | { type: 'Blue' };

const Red = { type: 'Red' as const };
const Green = { type: 'Green' as const };
const Blue = { type: 'Blue' as const };

const red = Red;

const green = Green;

const blue = Blue;

const colorName = (() => {
  const matchValue = red;
  if (matchValue.type === 'Red') {
    return "赤";
  }  if (matchValue.type === 'Green') {
    return "緑";
  }  if (matchValue.type === 'Blue') {
    return "青";
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

ssrgShow(colorName);

function colorToString(color: Color): string {
  return (() => {
  const matchValue = color;
  if (matchValue.type === 'Red') {
    return "赤色";
  }  if (matchValue.type === 'Green') {
    return "緑色";
  }  if (matchValue.type === 'Blue') {
    return "青色";
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

ssrgShow(colorToString(green));

ssrgPrint("--- 引数付きADT ---");

type Shape = { type: 'Circle', data: Array<number> } | { type: 'Rectangle', data: Array<number | number> };

function Circle(data0: number) { return { type: 'Circle' as const, data: [data0] }; }
function Rectangle(data0: number, data1: number) { return { type: 'Rectangle' as const, data: [data0, data1] }; }

const circle = Circle(5);

const rectangle = Rectangle(3, 4);

const area = (() => {
  const matchValue = circle;
  if (matchValue.type === 'Circle') {
    const radius = matchValue.data[0];
    return ((radius * radius) * 3.14);
  }  if (matchValue.type === 'Rectangle') {
    const width = matchValue.data[0];
    const height = matchValue.data[1];
    return (width * height);
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

ssrgShow(area);

function calculateArea(shape: Shape): number {
  return (() => {
  const matchValue = shape;
  if (matchValue.type === 'Circle') {
    const radius = matchValue.data[0];
    return ((radius * radius) * 3.14);
  }  if (matchValue.type === 'Rectangle') {
    const width = matchValue.data[0];
    const height = matchValue.data[1];
    return (width * height);
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

ssrgShow(calculateArea(circle));

ssrgShow(calculateArea(rectangle));

ssrgPrint("--- ビルトインMaybe型 ---");

const just42 = Just(42);

const nothing = Nothing;

const maybeValue = (() => {
  const matchValue = just42;
  if (matchValue.tag === 'Nothing') {
    return 0;
  }  if (matchValue.tag === 'Just') {
    const value = matchValue.value;
    return (value * 2);
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

ssrgShow(maybeValue);

function unwrapOr(maybe: Maybe<number>): (arg: number) => number {
  return function(defaultValue: number) {
      return (() => {
  const matchValue = maybe;
  if (matchValue.tag === 'Nothing') {
    return defaultValue;
  }  if (matchValue.tag === 'Just') {
    const value = matchValue.value;
    return value;
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
    };
}

ssrgShow(unwrapOr(just42)(0));

ssrgShow(unwrapOr(nothing)(0));

ssrgPrint("--- ビルトインEither型 ---");

const right100 = Right(100);

const leftError = Left("エラー");

const eitherValue = (() => {
  const matchValue = right100;
  if (matchValue.tag === 'Left') {
    const msg = matchValue.value;
    return 0;
  }  if (matchValue.tag === 'Right') {
    const value = matchValue.value;
    return (value + 10);
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

ssrgShow(eitherValue);

function handleResult(result: Either<string, number>): string {
  return (() => {
  const matchValue = result;
  if (matchValue.tag === 'Left') {
    const error = matchValue.value;
    return `エラー: ${error}`;
  }  if (matchValue.tag === 'Right') {
    const value = matchValue.value;
    return `成功: ${value}`;
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

ssrgShow(handleResult(right100));

ssrgShow(handleResult(leftError));

ssrgPrint("--- ビルトインList型 ---");

const myList = Cons(1, Cons(2, Cons(3, Empty)));

const emptyList = Empty;

const listHead = (() => {
  const matchValue = myList;
  if (matchValue.tag === 'Empty') {
    return -1;
  }  if (matchValue.tag === 'Cons') {
    const h = matchValue.head;
    const t = matchValue.tail;
    return h;
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

ssrgShow(listHead);

function listLength(list: List<number>): number {
  return (() => {
  const matchValue = list;
  if (matchValue.tag === 'Empty') {
    return 0;
  }  if (matchValue.tag === 'Cons') {
    const h = matchValue.head;
    const t = matchValue.tail;
    return (1 + listLength(t));
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

ssrgShow(listLength(myList));

ssrgShow(listLength(emptyList));

ssrgPrint("--- 複雑な例 ---");

type Result = { type: 'Success', data: Array<number> } | { type: 'Failure', data: Array<string> };

function Success(data0: number) { return { type: 'Success' as const, data: [data0] }; }
function Failure(data0: string) { return { type: 'Failure' as const, data: [data0] }; }

function processResult(result: Result): string {
  return (() => {
  const matchValue = result;
  if (matchValue.type === 'Success') {
    const value = matchValue.data[0];
    if ((value === 0)) {
      return "ゼロです";
    }
  }  if (matchValue.type === 'Success') {
    const value = matchValue.data[0];
    if ((value > 0)) {
      return `正の値: ${value}`;
    }
  }  if (matchValue.type === 'Success') {
    const value = matchValue.data[0];
    return `負の値: ${value}`;
  }  if (matchValue.type === 'Failure') {
    const msg = matchValue.data[0];
    return `失敗: ${msg}`;
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

const success = Success(42);

const zero = Success(0);

const failure = Failure("エラーが発生しました");

ssrgShow(processResult(success));

ssrgShow(processResult(zero));

ssrgShow(processResult(failure));

function classifyCharacter(char: string): string {
  return (() => {
  const matchValue = char;
  if ((matchValue === "a" || matchValue === "e" || matchValue === "i" || matchValue === "o" || matchValue === "u")) {
    return "母音";
  }  if (matchValue === "y") {
    return "半母音";
  }  if (true) {
    return "子音";
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

ssrgShow(classifyCharacter("a"));

ssrgShow(classifyCharacter("b"));

ssrgShow(classifyCharacter("y"));

function analyzeArray(arr: Array<number>): string {
  return (() => {
  const matchValue = arr;
  if (Array.isArray(matchValue) && matchValue.length === 0) {
    return "空の配列";
  }  if ((Array.isArray(matchValue) && matchValue.length === 1)) {
    const a = matchValue[0];
    return `要素1つ: ${a}`;
  }  if ((Array.isArray(matchValue) && matchValue.length === 2)) {
    const a = matchValue[0];
    const b = matchValue[1];
    return `要素2つ: ${a}, ${b}`;
  }  if ((Array.isArray(matchValue) && matchValue.length === 3)) {
    const a = matchValue[0];
    const b = matchValue[1];
    const c = matchValue[2];
    return `要素3つ: ${a}, ${b}, ${c}`;
  }  if (true) {
    return "4つ以上の要素";
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

ssrgShow(analyzeArray([]));

ssrgShow(analyzeArray([1]));

ssrgShow(analyzeArray([1, 2]));

ssrgShow(analyzeArray([1, 2, 3]));

ssrgShow(analyzeArray([1, 2, 3, 4]));

ssrgPrint("--- パターンマッチングの利点 ---");

ssrgPrint("パターンマッチングにより、型安全で表現力豊かなコードが書けます");

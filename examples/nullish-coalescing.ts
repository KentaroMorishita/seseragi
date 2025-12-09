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
  "value1": "Maybe<Int>",
  "value2": "Maybe<unknown>",
  "result1": "Int",
  "result2": "Int",
  "maybeString1": "Maybe<String>",
  "maybeString2": "Maybe<unknown>",
  "stringResult1": "String",
  "stringResult2": "String",
  "success": "Either<unknown, Int>",
  "failure": "Either<String, unknown>",
  "eitherResult1": "Int",
  "eitherResult2": "Int",
  "divResult1": "Int",
  "divResult2": "Int",
  "user1": "String",
  "user2": "String",
  "maybeA": "Maybe<Int>",
  "maybeB": "Maybe<Int>",
  "maybeC": "Maybe<Int>",
  "chainResult": "Int"
});

ssrgPrint("=== ??演算子（Nullish Coalescing）の使い方 ===");

ssrgPrint("--- Maybe型の例 ---");

const value1 = Just(42);

const value2 = Nothing;

const result1 = fromMaybe(0, value1);

const result2 = fromMaybe(0, value2);

ssrgShow(result1);

ssrgShow(result2);

const maybeString1 = Just("Hello");

const maybeString2 = Nothing;

const stringResult1 = fromMaybe("Default", maybeString1);

const stringResult2 = fromMaybe("Default", maybeString2);

ssrgShow(stringResult1);

ssrgShow(stringResult2);

ssrgPrint("--- Either型の例 ---");

const success = Right(100);

const failure = Left("error");

const eitherResult1 = fromRight(-1, success);

const eitherResult2 = fromRight(-1, failure);

ssrgShow(eitherResult1);

ssrgShow(eitherResult2);

ssrgPrint("--- 関数との組み合わせ ---");

function safeDivide(x: number): (arg: number) => Maybe<number> {
  return function(y: number) {
      return ((y === 0) ? Nothing : Just(Math.trunc(x / y)));
    };
}

const divResult1 = fromMaybe(0, safeDivide(10)(2));

const divResult2 = fromMaybe(0, safeDivide(10)(0));

ssrgShow(divResult1);

ssrgShow(divResult2);

function findUser(id: number): Maybe<string> {
  return ((id === 1) ? Just("Alice") : ((id === 2) ? Just("Bob") : Nothing));
}

const user1 = fromMaybe("Unknown", findUser(1));

const user2 = fromMaybe("Unknown", findUser(99));

ssrgShow(user1);

ssrgShow(user2);

ssrgPrint("--- チェーンされた例 ---");

const maybeA: Maybe<number> = Just(10);

const maybeB: Maybe<number> = Nothing;

const maybeC: Maybe<number> = Just(30);

const chainResult = ((maybeB.tag === 'Just' ? maybeB.value : (maybeC.tag === 'Just' ? maybeC.value : undefined)) ?? 0);

ssrgShow(chainResult);

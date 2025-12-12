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
  "someValue": "Maybe<Int>",
  "nothingValue": "Maybe<unknown>",
  "maybeNumber": "Maybe<Int>",
  "maybeString": "Maybe<String>",
  "maybeBoolean": "Maybe<Bool>",
  "empty": "Maybe<unknown>",
  "age1": "Maybe<Int>",
  "age2": "Maybe<Int>",
  "user1": "Maybe<String>",
  "user2": "Maybe<String>"
});

ssrgPrint("=== Maybe型の基本 ===");

ssrgPrint("--- Maybe型とは ---");

const someValue = Just(42);

const nothingValue = Nothing;

ssrgShow(someValue);

ssrgShow(nothingValue);

ssrgPrint("--- 基本的な操作 ---");

const maybeNumber = Just(100);

const maybeString = Just("Hello");

const maybeBoolean = Just(true);

const empty = Nothing;

ssrgShow(maybeNumber);

ssrgShow(maybeString);

ssrgShow(maybeBoolean);

ssrgShow(empty);

ssrgPrint("--- 安全な操作 ---");

function numberToMonth(n: number): Maybe<string> {
  return ((n === 1) ? Just("January") : ((n === 2) ? Just("February") : ((n === 3) ? Just("March") : ((n === 4) ? Just("April") : ((n === 5) ? Just("May") : ((n === 6) ? Just("June") : ((n === 7) ? Just("July") : ((n === 8) ? Just("August") : ((n === 9) ? Just("September") : ((n === 10) ? Just("October") : ((n === 11) ? Just("November") : ((n === 12) ? Just("December") : Nothing))))))))))));
}

ssrgShow(numberToMonth(0));

ssrgShow(numberToMonth(1));

ssrgShow(numberToMonth(12));

ssrgShow(numberToMonth(13));

function safeGet(index: number): Maybe<number> {
  return ((index < 0) ? Nothing : ((index >= 10) ? Nothing : Just((index * 10))));
}

ssrgShow(safeGet(2));

ssrgShow(safeGet(-1));

ssrgShow(safeGet(15));

ssrgPrint("--- 実用例 ---");

function parseAge(input: string): Maybe<number> {
  return ((input === "25") ? Just(25) : ((input === "30") ? Just(30) : Nothing));
}

const age1 = parseAge("25");

const age2 = parseAge("abc");

ssrgShow(age1);

ssrgShow(age2);

function findUser(id: number): Maybe<string> {
  return ((id === 1) ? Just("Alice") : ((id === 2) ? Just("Bob") : Nothing));
}

const user1 = findUser(1);

const user2 = findUser(99);

ssrgShow(user1);

ssrgShow(user2);

ssrgPrint("--- Maybe型の利点 ---");

ssrgPrint("Maybe型により、値の存在/非存在を型レベルで表現できます");

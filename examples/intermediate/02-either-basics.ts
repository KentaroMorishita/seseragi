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
  "successValue": "Either<unknown, Int>",
  "errorValue": "Either<String, unknown>",
  "successNumber": "Either<unknown, Int>",
  "successString": "Either<unknown, String>",
  "errorMessage": "Either<String, unknown>",
  "anotherError": "Either<String, unknown>",
  "result1": "Either<String, Int>",
  "result2": "Either<String, Int>",
  "user1": "Either<String, String>",
  "user2": "Either<String, String>"
});

ssrgPrint("=== Either型の基本 ===");

ssrgPrint("--- Either型とは ---");

const successValue = Right(42);

const errorValue = Left("Something went wrong");

ssrgShow(successValue);

ssrgShow(errorValue);

ssrgPrint("--- 基本的な操作 ---");

const successNumber = Right(100);

const successString = Right("Hello");

const errorMessage = Left("エラーが発生しました");

const anotherError = Left("別のエラー");

ssrgShow(successNumber);

ssrgShow(successString);

ssrgShow(errorMessage);

ssrgShow(anotherError);

ssrgPrint("--- 安全な操作 ---");

function safeDivide(x: number): (arg: number) => Either<string, number> {
  return function(y: number) {
      return ((y === 0) ? Left("ゼロ除算エラー") : Right(Math.trunc(x / y)));
    };
}

const result1 = safeDivide(10)(2);

const result2 = safeDivide(10)(0);

ssrgShow(result1);

ssrgShow(result2);

function parseInt(input: string): Either<string, number> {
  return ((input === "42") ? Right(42) : ((input === "100") ? Right(100) : Left(`"${input}"は数値ではありません`)));
}

ssrgShow(parseInt("42"));

ssrgShow(parseInt("abc"));

ssrgPrint("--- 実用例 ---");

function findUser(id: number): Either<string, string> {
  return ((id === 1) ? Right("Alice") : ((id === 2) ? Right("Bob") : Left(`ユーザーID ${id} は存在しません`)));
}

const user1 = findUser(1);

const user2 = findUser(99);

ssrgShow(user1);

ssrgShow(user2);

function readFile(filename: string): Either<string, string> {
  return ((filename === "config.txt") ? Right("設定内容") : ((filename === "data.txt") ? Right("データ内容") : Left(`ファイル "${filename}" が見つかりません`)));
}

ssrgShow(readFile("config.txt"));

ssrgShow(readFile("missing.txt"));

ssrgPrint("--- バリデーション例 ---");

function validateAge(age: number): Either<string, number> {
  return ((age < 0) ? Left("年齢は負数にできません") : ((age > 150) ? Left("年齢が無効です") : Right(age)));
}

ssrgShow(validateAge(25));

ssrgShow(validateAge(-5));

ssrgShow(validateAge(200));

ssrgPrint("--- Either型の利点 ---");

ssrgPrint("Either型により、エラーを値として扱えます");

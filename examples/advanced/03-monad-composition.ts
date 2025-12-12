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
  "maybeX": "Maybe<Int>",
  "maybeY": "Maybe<Int>",
  "maybeZ": "Maybe<unknown>",
  "result1": "Maybe<Int>",
  "result2": "Maybe<Int>",
  "result3": "Maybe<Int>",
  "result4": "Maybe<Int>",
  "rightX": "Either<unknown, Int>",
  "rightY": "Either<unknown, Int>",
  "leftError": "Either<String, unknown>",
  "eitherResult1": "Either<unknown, Int>",
  "eitherResult2": "Either<unknown, Int>",
  "eitherResult3": "Either<String, Int>",
  "eitherResult4": "Either<unknown, Int>",
  "eitherResult5": "Either<unknown, Int>",
  "accessResult1": "Maybe<String>",
  "accessResult2": "Maybe<String>",
  "accessResult3": "Maybe<String>",
  "userResult1": "Either<unknown, User>",
  "userResult2": "Either<unknown, User>",
  "userResult3": "Either<unknown, User>",
  "compose": "function",
  "doubleAndIncrement": "function",
  "squareAndDouble": "function",
  "report1": "Either<String, String>",
  "report2": "Either<String, String>",
  "report3": "Either<String, String>"
});

class User {
  name: string;
  age: number;
  email: string;

  constructor(fields: { name: string, age: number, email: string }) {
    this.name = fields.name;
    this.age = fields.age;
    this.email = fields.email;
  }
}

ssrgPrint("=== モナドの合成 ===");

ssrgPrint("--- Maybe値の合成 ---");

const maybeX = Just(10);

const maybeY = Just(20);

const maybeZ = Nothing;

function add(x: number): (arg: number) => number {
  return function(y: number) {
      return (x + y);
    };
}

const result1 = applyMaybe(applyMaybe(Just(add), maybeX), maybeY);

const result2 = applyMaybe(applyMaybe(Just(add), maybeX), maybeZ);

ssrgShow(result1);

ssrgShow(result2);

function doubleIfPositive(x: number): Maybe<number> {
  return ((x > 0) ? Just((x * 2)) : Nothing);
}

function addTen(x: number): Maybe<number> {
  return ((x < 100) ? Just((x + 10)) : Nothing);
}

const result3 = bindMaybe(bindMaybe(maybeX, doubleIfPositive), addTen);

const result4 = bindMaybe(bindMaybe(maybeZ, doubleIfPositive), addTen);

ssrgShow(result3);

ssrgShow(result4);

ssrgPrint("--- Either値の合成 ---");

const rightX = Right(15);

const rightY = Right(25);

const leftError = Left("エラーが発生しました");

function multiply(x: number): (arg: number) => number {
  return function(y: number) {
      return (x * y);
    };
}

const eitherResult1 = applyEither(applyEither(Right(multiply), rightX), rightY);

const eitherResult2 = applyEither(applyEither(Right(multiply), rightX), leftError);

ssrgShow(eitherResult1);

ssrgShow(eitherResult2);

function multiplyBy2(x: number): Either<string, number> {
  return ((x > 0) ? Right((x * 2)) : Left("値は正数である必要があります"));
}

function subtractFive(x: number): Either<string, number> {
  return ((x >= 5) ? Right((x - 5)) : Left("値が小さすぎます"));
}

const eitherResult3 = bindEither(bindEither(rightX, multiplyBy2), subtractFive);

const eitherResult4 = bindEither(bindEither(Right(0), multiplyBy2), subtractFive);

const eitherResult5 = bindEither(bindEither(Right(2), multiplyBy2), subtractFive);

ssrgShow(eitherResult3);

ssrgShow(eitherResult4);

ssrgShow(eitherResult5);

ssrgPrint("--- 複雑な計算のチェーン ---");

function parseUserId(input: string): Maybe<number> {
  return (() => {
  const matchValue = input;
  if (matchValue === "user123") {
    return Just(123);
  }  if (matchValue === "admin456") {
    return Just(456);
  }  if (matchValue === "guest789") {
    return Just(789);
  }  if (true) {
    return Nothing;
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

function checkPermission(userId: number): Maybe<string> {
  return (() => {
  const matchValue = userId;
  if (matchValue === 123) {
    return Just("read");
  }  if (matchValue === 456) {
    return Just("admin");
  }  if (matchValue === 789) {
    return Just("guest");
  }  if (true) {
    return Nothing;
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

function accessResource(permission: string): Maybe<string> {
  return (() => {
  const matchValue = permission;
  if (matchValue === "read") {
    return Just("データを読み取りました");
  }  if (matchValue === "admin") {
    return Just("管理者権限でアクセスしました");
  }  if (matchValue === "guest") {
    return Just("ゲストとしてアクセスしました");
  }  if (true) {
    return Nothing;
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

const accessResult1 = bindMaybe(bindMaybe(bindMaybe(Just("user123"), parseUserId), checkPermission), accessResource);

const accessResult2 = bindMaybe(bindMaybe(bindMaybe(Just("admin456"), parseUserId), checkPermission), accessResource);

const accessResult3 = bindMaybe(bindMaybe(bindMaybe(Just("unknown"), parseUserId), checkPermission), accessResource);

ssrgShow(accessResult1);

ssrgShow(accessResult2);

ssrgShow(accessResult3);

ssrgPrint("--- バリデーションの合成 ---");

function validateAge(age: number): Either<string, number> {
  return (() => {
  const matchValue = age;
  if (true) {
    const n = matchValue;
    if ((n < 0)) {
      return Left("年齢は負数にできません");
    }
  }  if (true) {
    const n = matchValue;
    if ((n > 150)) {
      return Left("年齢が無効です");
    }
  }  if (true) {
    const n = matchValue;
    return Right(n);
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

function validateName(name: string): Either<string, string> {
  return (() => {
  const matchValue = name;
  if (matchValue === "") {
    return Left("名前が空です");
  }  if (matchValue === "admin") {
    return Left("予約語は使用できません");
  }  if (matchValue === "root") {
    return Left("予約語は使用できません");
  }  if (true) {
    const n = matchValue;
    return Right(n);
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

function validateEmail(email: string): Either<string, string> {
  return (() => {
  const matchValue = email;
  if (matchValue === "") {
    return Left("メールアドレスが空です");
  }  if (true) {
    const e = matchValue;
    if ((e === "test@test.com")) {
      return Left("テストアドレスは使用できません");
    }
  }  if (true) {
    const e = matchValue;
    return Right(e);
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

function createUser(name: string): (arg: number) => (arg: string) => User {
  return function(age: number) {
      return function(email: string) {
      return new User({ name: name, age: age, email: email });
    };
    };
}

const userResult1 = applyEither(applyEither(applyEither(Right(createUser), validateName("Alice")), validateAge(25)), validateEmail("alice@example.com"));

const userResult2 = applyEither(applyEither(applyEither(Right(createUser), validateName("admin")), validateAge(25)), validateEmail("alice@example.com"));

const userResult3 = applyEither(applyEither(applyEither(Right(createUser), validateName("Bob")), validateAge(-5)), validateEmail("test@test.com"));

ssrgShow(userResult1);

ssrgShow(userResult2);

ssrgShow(userResult3);

ssrgPrint("--- 関数の合成 ---");

const compose = (f: any) => (g: any) => (x: any) => pipe(g(x), f);

function double(x: number): number {
  return (x * 2);
}

function increment(x: number): number {
  return (x + 1);
}

function square(x: number): number {
  return (x * x);
}

const doubleAndIncrement = compose(increment)(double);

const squareAndDouble = compose(double)(square);

ssrgShow(doubleAndIncrement(5));

ssrgShow(squareAndDouble(3));

ssrgPrint("--- データ処理パイプライン ---");

function fetchUserData(userId: string): Either<string, string> {
  return (() => {
  const matchValue = userId;
  if (matchValue === "user123") {
    return Right(`{"id": "user123", "score": 85}`);
  }  if (matchValue === "user456") {
    return Right(`{"id": "user456", "score": -10}`);
  }  if (true) {
    return Left(`ユーザー ${userId} が見つかりません`);
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

function parseScore(json: string): Either<string, number> {
  return (() => {
  const matchValue = json;
  if (true) {
    const s = matchValue;
    if ((s === `{"id": "user123", "score": 85}`)) {
      return Right(85);
    }
  }  if (true) {
    const s = matchValue;
    if ((s === `{"id": "user456", "score": -10}`)) {
      return Right(-10);
    }
  }  if (true) {
    return Left("JSONの解析に失敗しました");
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

function calculateBonus(score: number): Either<string, number> {
  return (() => {
  const matchValue = score;
  if (true) {
    const s = matchValue;
    if ((s >= 80)) {
      return Right((s * 2));
    }
  }  if (true) {
    const s = matchValue;
    if ((s >= 60)) {
      return Right((s + 20));
    }
  }  if (true) {
    const s = matchValue;
    if ((s > 0)) {
      return Right(s);
    }
  }  if (true) {
    return Left(`スコア ${score} は無効です（負の値）`);
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

function generateReport(bonus: number): Either<string, string> {
  return (() => {
  const matchValue = bonus;
  if (true) {
    const b = matchValue;
    if ((b >= 150)) {
      return Right(`優秀！ボーナススコア: ${b}`);
    }
  }  if (true) {
    const b = matchValue;
    if ((b >= 80)) {
      return Right(`良好。ボーナススコア: ${b}`);
    }
  }  if (true) {
    const b = matchValue;
    return Right(`ボーナススコア: ${b}`);
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

const report1 = bindEither(bindEither(bindEither(fetchUserData("user123"), parseScore), calculateBonus), generateReport);

const report2 = bindEither(bindEither(bindEither(fetchUserData("user456"), parseScore), calculateBonus), generateReport);

const report3 = bindEither(bindEither(bindEither(fetchUserData("unknown"), parseScore), calculateBonus), generateReport);

ssrgShow(report1);

ssrgShow(report2);

ssrgShow(report3);

ssrgPrint("--- モナド合成の利点 ---");

ssrgPrint("モナドの合成により、複雑な計算を安全かつ簡潔に表現できます");

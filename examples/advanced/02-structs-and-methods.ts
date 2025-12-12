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
    "Vector": {
      "add": __ssrg_Vector_f4pl4mu_add,
      "subtract": __ssrg_Vector_f4pl4mu_subtract,
      "scale": __ssrg_Vector_f4pl4mu_scale,
      "distanceSquared": __ssrg_Vector_f4pl4mu_distanceSquared
    },
    "Account": {
      "getInfo": __ssrg_Account_f4pl4mu_getInfo,
      "deposit": __ssrg_Account_f4pl4mu_deposit,
      "withdraw": __ssrg_Account_f4pl4mu_withdraw
    },
  };

  // Initialize operator dispatch table
  __structOperators = {
    "Vector": {
      "+": __ssrg_Vector_f4pl4mu_op_add,
      "-": __ssrg_Vector_f4pl4mu_op_sub,
      "*": __ssrg_Vector_f4pl4mu_op_mul
    },
    "Account": {

    },
  };

})();

// 変数型情報テーブルの初期化
Object.assign(__variableTypes, {
  "alice": "Person { age: Int, name: String }",
  "bob": "Person { age: Int, name: String }",
  "origin": "Point { x: Float, y: Float }",
  "point1": "Point { x: Float, y: Float }",
  "point2": "Point { x: Float, y: Float }",
  "aliceName": "String",
  "aliceAge": "Int",
  "x1": "Float",
  "y1": "Float",
  "v1": "Vector { x: Int, y: Int }",
  "v2": "Vector { x: Int, y: Int }",
  "sum": "Vector",
  "diff": "Vector",
  "scaled": "Vector",
  "distance": "Int"
});

class Person {
  name: string;
  age: number;

  constructor(fields: { name: string, age: number }) {
    this.name = fields.name;
    this.age = fields.age;
  }
}

class Point {
  x: number;
  y: number;

  constructor(fields: { x: number, y: number }) {
    this.x = fields.x;
    this.y = fields.y;
  }
}

class Employee {
  id: number;
  name: string;
  department: string;
  salary: number;
  isActive: boolean;

  constructor(fields: { id: number, name: string, department: string, salary: number, isActive: boolean }) {
    this.id = fields.id;
    this.name = fields.name;
    this.department = fields.department;
    this.salary = fields.salary;
    this.isActive = fields.isActive;
  }
}

class Vector {
  x: number;
  y: number;

  constructor(fields: { x: number, y: number }) {
    this.x = fields.x;
    this.y = fields.y;
  }
}

class Account {
  id: number;
  owner: string;
  balance: number;

  constructor(fields: { id: number, owner: string, balance: number }) {
    this.id = fields.id;
    this.owner = fields.owner;
    this.balance = fields.balance;
  }
}

// Vector implementation
function __ssrg_Vector_f4pl4mu_add(self: A, other: Vector): Vector {
  return (() => {
  const x = (self.x + other.x);
  const y = (self.y + other.y);
  return (() => { const __tmpi3zitw = { x: x, y: y }; return Object.assign(Object.create(Vector.prototype), __tmpi3zitw); })();
})();
}
function __ssrg_Vector_f4pl4mu_subtract(self: B, other: Vector): Vector {
  return (() => {
  const x = (self.x - other.x);
  const y = (self.y - other.y);
  return (() => { const __tmp093hg3 = { x: x, y: y }; return Object.assign(Object.create(Vector.prototype), __tmp093hg3); })();
})();
}
function __ssrg_Vector_f4pl4mu_scale(self: C, factor: number): Vector {
  return (() => {
  const x = (self.x * factor);
  const y = (self.y * factor);
  return (() => { const __tmptq3tl0 = { x: x, y: y }; return Object.assign(Object.create(Vector.prototype), __tmptq3tl0); })();
})();
}
function __ssrg_Vector_f4pl4mu_distanceSquared(self: T3): number {
  return ((self.x * self.x) + (self.y * self.y));
}
function __ssrg_Vector_f4pl4mu_op_add(self: T4, other: T5): Vector {
  return __dispatchMethod(self, "add", other);
}
function __ssrg_Vector_f4pl4mu_op_sub(self: T6, other: Vector): Vector {
  return __dispatchMethod(self, "subtract", other);
}
function __ssrg_Vector_f4pl4mu_op_mul(self: T7, factor: number): Vector {
  return __dispatchMethod(self, "scale", factor);
}

// Account implementation
function __ssrg_Account_f4pl4mu_getInfo(self: T8): string {
  return (() => {
  const { id, owner, balance } = self;
  return `口座ID: ${id}, 所有者: ${owner}, 残高: ${balance}`;
})();
}
function __ssrg_Account_f4pl4mu_deposit(self: T9, amount: number): Account {
  return (() => {
  return (() => { const __tmpasg1py = { ...self, balance: (self.balance + amount) }; return Object.assign(Object.create(Account.prototype), __tmpasg1py); })();
})();
}
function __ssrg_Account_f4pl4mu_withdraw(self: T10, amount: number): Either<string, Account> {
  return (() => {
  const balance = (self.balance - amount);
  return ((amount <= balance) ? Right((() => { const __tmpmc652j = { ...self, balance: balance }; return Object.assign(Object.create(Account.prototype), __tmpmc652j); })()) : Left(`残高不足です。現在の残高: ${self.balance}, 出金額: ${amount}`));
})();
}

ssrgPrint("=== 構造体とメソッド ===");

ssrgPrint("--- 構造体の定義 ---");

ssrgPrint("--- 構造体のインスタンス化 ---");

const alice = new Person({ name: "Alice", age: 30 });

const bob = new Person({ name: "Bob", age: 25 });

ssrgShow(alice);

ssrgShow(bob);

const origin = new Point({ x: 0, y: 0 });

const point1 = new Point({ x: 3, y: 4 });

const point2 = new Point({ x: 1, y: 2 });

ssrgShow(origin);

ssrgShow(point1);

ssrgShow(point2);

ssrgPrint("--- フィールドアクセス ---");

const aliceName = alice.name;

const aliceAge = alice.age;

ssrgShow(aliceName);

ssrgShow(aliceAge);

const x1 = point1.x;

const y1 = point1.y;

ssrgShow(x1);

ssrgShow(y1);

ssrgPrint("--- 構造体のメソッド実装 ---");

const v1 = new Vector({ x: 3, y: 4 });

const v2 = new Vector({ x: 1, y: 2 });

ssrgShow(v1);

ssrgShow(v2);

ssrgPrint("--- メソッドの呼び出し ---");

const sum = __dispatchMethod(v1, "add", v2);

const diff = __dispatchMethod(v1, "subtract", v2);

const scaled = __dispatchMethod(v1, "scale", 3);

const distance = __dispatchMethod(v1, "distanceSquared");

ssrgShow(sum);

ssrgShow(diff);

ssrgShow(scaled);

ssrgShow(distance);

ssrgShow(__dispatchOperator(v1, "+", v2));

ssrgShow(__dispatchOperator(v1, "-", v2));

ssrgShow(__dispatchOperator(v1, "*", 3));

ssrgPrint("--- 実用的な例 ---");

const account = new Account({ id: 1001, owner: "Alice", balance: 10000 });

ssrgShow(account);

const account_prime = __dispatchMethod(account, "deposit", 5000);

ssrgShow(account_prime);

const withdrawResult = __dispatchMethod(account_prime, "withdraw", 3000);

ssrgShow(withdrawResult);

const failedWithdraw = __dispatchMethod(account_prime, "withdraw", 20000);

ssrgShow(failedWithdraw);

const account_prime_prime = (() => {
  const matchValue = withdrawResult;
  if (matchValue.tag === 'Right') {
    const acc = matchValue.value;
    return acc;
  }  if (matchValue.tag === 'Left') {
    return account_prime;
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

ssrgShow(__dispatchMethod(account_prime_prime, "getInfo"));

ssrgPrint("--- 構造体の利点 ---");

ssrgPrint("構造体により、関連するデータとメソッドを一箇所にまとめられます");

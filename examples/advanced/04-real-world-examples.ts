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
    "Product": {
      "updateStock": __ssrg_Product_f4pl4mu_updateStock,
      "sell": __ssrg_Product_f4pl4mu_sell,
      "restock": __ssrg_Product_f4pl4mu_restock
    },
    "BankAccount": {
      "deposit": __ssrg_BankAccount_f4pl4mu_deposit,
      "withdraw": __ssrg_BankAccount_f4pl4mu_withdraw,
      "getStatement": __ssrg_BankAccount_f4pl4mu_getStatement
    },
    "Student": {
      "addGrade": __ssrg_Student_f4pl4mu_addGrade,
      "getAverageScore": __ssrg_Student_f4pl4mu_getAverageScore
    },
  };

  // Initialize operator dispatch table
  __structOperators = {
    "Product": {

    },
    "BankAccount": {

    },
    "Student": {

    },
  };

})();

// 変数型情報テーブルの初期化
Object.assign(__variableTypes, {
  "result1": "Either<String, Int>",
  "result2": "Either<String, Int>",
  "result3": "Either<String, Int>",
  "apple": "Product { id: Int, name: String, price: Int, stock: Int }",
  "soldApple": "Either<String, Product>",
  "restockedApple": "unknown",
  "account": "BankAccount { balance: Int, id: Int, owner: String }",
  "step1": "Either<String, BankAccount>",
  "step2": "unknown",
  "step3": "unknown",
  "statement": "unknown",
  "failedWithdraw": "Either<String, BankAccount>",
  "invalidDeposit": "Either<String, BankAccount>",
  "mathGrade": "Grade { score: Int, subject: String }",
  "englishGrade": "Grade { score: Int, subject: String }",
  "student": "Student { grades: List<Grade>, name: String }",
  "student'": "Student",
  "average": "Maybe<Int>"
});

class Product {
  id: number;
  name: string;
  price: number;
  stock: number;

  constructor(fields: { id: number, name: string, price: number, stock: number }) {
    this.id = fields.id;
    this.name = fields.name;
    this.price = fields.price;
    this.stock = fields.stock;
  }
}

class BankAccount {
  id: number;
  owner: string;
  balance: number;

  constructor(fields: { id: number, owner: string, balance: number }) {
    this.id = fields.id;
    this.owner = fields.owner;
    this.balance = fields.balance;
  }
}

class Grade {
  subject: string;
  score: number;

  constructor(fields: { subject: string, score: number }) {
    this.subject = fields.subject;
    this.score = fields.score;
  }
}

class Student {
  name: string;
  grades: List<Grade>;

  constructor(fields: { name: string, grades: List<Grade> }) {
    this.name = fields.name;
    this.grades = fields.grades;
  }
}

// Product implementation
function __ssrg_Product_f4pl4mu_updateStock(self: A, newStock: number): Product {
  return (() => { const __tmpibwt2l = { ...self, stock: newStock }; return Object.assign(Object.create(Product.prototype), __tmpibwt2l); })();
}
function __ssrg_Product_f4pl4mu_sell(self: B, quantity: number): Either<string, Product> {
  return (() => {
  const matchValue = quantity;
  if (true) {
    const q = matchValue;
    if ((q <= 0)) {
      return Left("数量は正数である必要があります");
    }
  }  if (true) {
    const q = matchValue;
    if ((q > self.stock)) {
      return Left("在庫が不足しています");
    }
  }  if (true) {
    const q = matchValue;
    return Right(__dispatchMethod(self, "updateStock", (self.stock - q)));
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}
function __ssrg_Product_f4pl4mu_restock(self: C, quantity: number): Either<string, Product> {
  return (() => {
  const matchValue = quantity;
  if (true) {
    const q = matchValue;
    if ((q <= 0)) {
      return Left("数量は正数である必要があります");
    }
  }  if (true) {
    const q = matchValue;
    return Right(__dispatchMethod(self, "updateStock", (self.stock + q)));
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

// BankAccount implementation
function __ssrg_BankAccount_f4pl4mu_deposit(self: T3, amount: number): Either<string, BankAccount> {
  return (() => {
  const matchValue = amount;
  if (true) {
    const a = matchValue;
    if ((a <= 0)) {
      return Left("入金額は正数である必要があります");
    }
  }  if (true) {
    const a = matchValue;
    return Right((() => { const __tmp7rbeuq = { ...self, balance: (self.balance + a) }; return Object.assign(Object.create(BankAccount.prototype), __tmp7rbeuq); })());
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}
function __ssrg_BankAccount_f4pl4mu_withdraw(self: T4, amount: number): Either<string, BankAccount> {
  return (() => {
  const matchValue = amount;
  if (true) {
    const a = matchValue;
    if ((a <= 0)) {
      return Left("出金額は正数である必要があります");
    }
  }  if (true) {
    const a = matchValue;
    if ((a > self.balance)) {
      return Left("残高が不足しています");
    }
  }  if (true) {
    const a = matchValue;
    return Right((() => { const __tmpmylfat = { ...self, balance: (self.balance - a) }; return Object.assign(Object.create(BankAccount.prototype), __tmpmylfat); })());
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}
function __ssrg_BankAccount_f4pl4mu_getStatement(self: T5): string {
  return (() => {
  const { id, owner, balance } = self;
  return `口座ID: ${id}, 所有者: ${owner}, 残高: ${balance}円`;
})();
}

// Student implementation
function __ssrg_Student_f4pl4mu_addGrade(self: T6, grade: Grade): Student {
  return (() => {
  const grades = Cons(grade, self.grades);
  return (() => { const __tmp2ybmmp = { ...self, grades: grades }; return Object.assign(Object.create(Student.prototype), __tmp2ybmmp); })();
})();
}
function __ssrg_Student_f4pl4mu_getAverageScore(self: T7): Maybe<number> {
  return (() => {
  const matchValue = self.grades;
  if (matchValue.tag === 'Empty') {
    return Nothing;
  }  if (true) {
    return Just(calculateAverage(self.grades)(0)(0));
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

ssrgPrint("=== 実践的な例 ===");

ssrgPrint("--- FizzBuzz ---");

function fizzBuzz(n: number): string {
  return (() => {
  const matchValue = { tag: 'Tuple', elements: [(n % 3), (n % 5)] };
  if (matchValue.elements[0] === 0 && matchValue.elements[1] === 0) {
    return "FizzBuzz";
  }  if (matchValue.elements[0] === 0 && true) {
    return "Fizz";
  }  if (true && matchValue.elements[1] === 0) {
    return "Buzz";
  }  if (true) {
    return `${n}`;
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}

Array.from({length: 100 - 1 + 1}, (_, i) => i + 1).map(n => ssrgPrint(fizzBuzz(n)));

ssrgPrint("--- 計算器 ---");

type Operation = { type: 'Add' } | { type: 'Subtract' } | { type: 'Multiply' } | { type: 'Divide' };

const Add = { type: 'Add' as const };
const Subtract = { type: 'Subtract' as const };
const Multiply = { type: 'Multiply' as const };
const Divide = { type: 'Divide' as const };

function calculate(op: Operation): (arg: number) => (arg: number) => Either<string, number> {
  return function(x: number) {
      return function(y: number) {
      return (() => {
  const matchValue = op;
  if (matchValue.type === 'Add') {
    return Right((x + y));
  }  if (matchValue.type === 'Subtract') {
    return Right((x - y));
  }  if (matchValue.type === 'Multiply') {
    return Right((x * y));
  }  if (matchValue.type === 'Divide') {
    return (() => {
  const matchValue = y;
  if (matchValue === 0) {
    return Left("ゼロ除算エラー");
  }  if (true) {
    return Right(Math.trunc(x / y));
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
    };
    };
}

const result1 = calculate(Add)(10)(5);

const result2 = calculate(Divide)(20)(4);

const result3 = calculate(Divide)(10)(0);

ssrgShow(result1);

ssrgShow(result2);

ssrgShow(result3);

ssrgPrint("--- 在庫管理システム ---");

const apple = new Product({ id: 1, name: "りんご", price: 100, stock: 50 });

ssrgShow(apple);

const soldApple = __dispatchMethod(apple, "sell", 10);

ssrgShow(soldApple);

const restockedApple = (() => {
  const matchValue = soldApple;
  if (matchValue.tag === 'Left') {
    const error = matchValue.value;
    return Left(error);
  }  if (matchValue.tag === 'Right') {
    const product = matchValue.value;
    return __dispatchMethod(product, "restock", 20);
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

ssrgShow(restockedApple);

ssrgPrint("--- 銀行システム ---");

type Transaction = { type: 'Deposit', data: Array<number> } | { type: 'Withdraw', data: Array<number> } | { type: 'Transfer', data: Array<number | number> };

function Deposit(data0: number) { return { type: 'Deposit' as const, data: [data0] }; }
function Withdraw(data0: number) { return { type: 'Withdraw' as const, data: [data0] }; }
function Transfer(data0: number, data1: number) { return { type: 'Transfer' as const, data: [data0, data1] }; }

const account = new BankAccount({ id: 1001, owner: "田中太郎", balance: 50000 });

ssrgShow(account);

const step1 = __dispatchMethod(account, "deposit", 20000);

const step2 = (() => {
  const matchValue = step1;
  if (matchValue.tag === 'Left') {
    const error = matchValue.value;
    return Left(error);
  }  if (matchValue.tag === 'Right') {
    const acc = matchValue.value;
    return __dispatchMethod(acc, "withdraw", 15000);
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

const step3 = (() => {
  const matchValue = step2;
  if (matchValue.tag === 'Left') {
    const error = matchValue.value;
    return Left(error);
  }  if (matchValue.tag === 'Right') {
    const acc = matchValue.value;
    return __dispatchMethod(acc, "deposit", 5000);
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

ssrgShow(step1);

ssrgShow(step2);

ssrgShow(step3);

const statement = (() => {
  const matchValue = step3;
  if (matchValue.tag === 'Left') {
    const error = matchValue.value;
    return error;
  }  if (matchValue.tag === 'Right') {
    const acc = matchValue.value;
    return __dispatchMethod(acc, "getStatement");
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();

ssrgShow(statement);

const failedWithdraw = __dispatchMethod(account, "withdraw", 100000);

ssrgShow(failedWithdraw);

const invalidDeposit = __dispatchMethod(account, "deposit", -1000);

ssrgShow(invalidDeposit);

ssrgPrint("--- 成績管理システム ---");

function calculateAverage(grades: List<Grade>): (arg: number) => (arg: number) => number {
  return function(sum: number) {
      return function(count: number) {
      return (() => {
  const matchValue = grades;
  if (matchValue.tag === 'Empty') {
    return Math.trunc(sum / count);
  }  if (matchValue.tag === 'Cons') {
    const grade = matchValue.head;
    const rest = matchValue.tail;
    return calculateAverage(rest)((sum + grade.score))((count + 1));
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
    };
    };
}

const mathGrade = new Grade({ subject: "数学", score: 80 });

const englishGrade = new Grade({ subject: "英語", score: 90 });

const student = new Student({ name: "佐藤花子", grades: Cons(mathGrade, Empty) });

ssrgShow(student);

const student_prime = __dispatchMethod(student, "addGrade", englishGrade);

ssrgShow(student_prime);

const average = __dispatchMethod(student_prime, "getAverageScore");

ssrgShow(average);

ssrgPrint("--- 実践的なプログラミングの原則 ---");

ssrgPrint("実践的なプログラムでは、型安全性とエラーハンドリングが重要です");

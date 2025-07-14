// Generated TypeScript code from Seseragi

// Seseragi runtime helpers

type Maybe<T> = { tag: 'Just'; value: T } | { tag: 'Nothing' };
type Either<L, R> = { tag: 'Left'; value: L } | { tag: 'Right'; value: R };
type List<T> = { tag: 'Empty' } | { tag: 'Cons'; head: T; tail: List<T> };

const curry = (fn: Function) => {
  return function curried(...args: any[]) {
    if (args.length >= fn.length) {
      return fn.apply(this, args);
    } else {
      return function(...args2: any[]) {
        return curried.apply(this, args.concat(args2));
      };
    }
  };
};

const pipe = <T, U>(value: T, fn: (arg: T) => U): U => fn(value);

const reversePipe = <T, U>(fn: (arg: T) => U, value: T): U => fn(value);

const map = <T, U>(fn: (value: T) => U, container: Maybe<T> | Either<any, T>): Maybe<U> | Either<any, U> => {
  if ('tag' in container) {
    if (container.tag === 'Just') return Just(fn(container.value));
    if (container.tag === 'Right') return Right(fn(container.value));
    if (container.tag === 'Nothing') return Nothing;
    if (container.tag === 'Left') return container;
  }
  return Nothing;
};

const applyWrapped = <T, U>(wrapped: Maybe<(value: T) => U> | Either<any, (value: T) => U>, container: Maybe<T> | Either<any, T>): Maybe<U> | Either<any, U> => {
  // Maybe types
  if (wrapped.tag === 'Nothing' || container.tag === 'Nothing') return Nothing;
  if (wrapped.tag === 'Just' && container.tag === 'Just') return Just(wrapped.value(container.value));
  // Either types
  if (wrapped.tag === 'Left') return wrapped;
  if (container.tag === 'Left') return container;
  if (wrapped.tag === 'Right' && container.tag === 'Right') return Right(wrapped.value(container.value));
  return Nothing;
};

const bind = <T, U>(container: Maybe<T> | Either<any, T>, fn: (value: T) => Maybe<U> | Either<any, U>): Maybe<U> | Either<any, U> => {
  if (container.tag === 'Just') return fn(container.value);
  if (container.tag === 'Right') return fn(container.value);
  if (container.tag === 'Nothing') return Nothing;
  if (container.tag === 'Left') return container;
  return Nothing;
};

const foldMonoid = <T>(arr: T[], empty: T, combine: (a: T, b: T) => T): T => {
  return arr.reduce(combine, empty);
};

// Array monadic functions
const mapArray = <T, U>(fa: T[], f: (a: T) => U): U[] => {
  return fa.map(f);
};

const applyArray = <T, U>(ff: ((a: T) => U)[], fa: T[]): U[] => {
  const result: U[] = [];
  for (const func of ff) {
    for (const value of fa) {
      result.push(func(value));
    }
  }
  return result;
};

const bindArray = <T, U>(ma: T[], f: (value: T) => U[]): U[] => {
  const result: U[] = [];
  for (const value of ma) {
    result.push(...f(value));
  }
  return result;
};

// List monadic functions
const mapList = <T, U>(fa: any, f: (a: T) => U): any => {
  if (fa.tag === 'Empty') return { tag: 'Empty' };
  return { tag: 'Cons', head: f(fa.head), tail: mapList(fa.tail, f) };
};

const applyList = <T, U>(ff: any, fa: any): any => {
  if (ff.tag === 'Empty') return { tag: 'Empty' };
  const mappedValues = mapList(fa, ff.head);
  const restApplied = applyList(ff.tail, fa);
  return concatList(mappedValues, restApplied);
};

const concatList = <T>(list1: any, list2: any): any => {
  if (list1.tag === 'Empty') return list2;
  return { tag: 'Cons', head: list1.head, tail: concatList(list1.tail, list2) };
};

const bindList = <T, U>(ma: any, f: (value: T) => any): any => {
  if (ma.tag === 'Empty') return { tag: 'Empty' };
  const headResult = f(ma.head);
  const tailResult = bindList(ma.tail, f);
  return concatList(headResult, tailResult);
};

// Maybe monadic functions
const mapMaybe = <T, U>(fa: Maybe<T>, f: (a: T) => U): Maybe<U> => {
  return fa.tag === 'Just' ? Just(f(fa.value)) : Nothing;
};

const applyMaybe = <T, U>(ff: Maybe<(a: T) => U>, fa: Maybe<T>): Maybe<U> => {
  return ff.tag === 'Just' && fa.tag === 'Just' ? Just(ff.value(fa.value)) : Nothing;
};

const bindMaybe = <T, U>(ma: Maybe<T>, f: (value: T) => Maybe<U>): Maybe<U> => {
  return ma.tag === 'Just' ? f(ma.value) : Nothing;
};

// Either monadic functions
const mapEither = <L, R, U>(ea: Either<L, R>, f: (value: R) => U): Either<L, U> => {
  return ea.tag === 'Right' ? Right(f(ea.value)) : ea;
};

const applyEither = <L, R, U>(ef: Either<L, (value: R) => U>, ea: Either<L, R>): Either<L, U> => {
  return ef.tag === 'Right' && ea.tag === 'Right' ? Right(ef.value(ea.value)) :
         ef.tag === 'Left' ? ef : ea;
};

const bindEither = <L, R, U>(ea: Either<L, R>, f: (value: R) => Either<L, U>): Either<L, U> => {
  return ea.tag === 'Right' ? f(ea.value) : ea;
};

const Just = <T>(value: T): Maybe<T> => ({ tag: 'Just', value });
const Nothing: Maybe<never> = { tag: 'Nothing' };

const Left = <L>(value: L): Either<L, never> => ({ tag: 'Left', value });
const Right = <R>(value: R): Either<never, R> => ({ tag: 'Right', value });

const Empty: List<never> = { tag: 'Empty' };
const Cons = <T>(head: T, tail: List<T>): List<T> => ({ tag: 'Cons', head, tail });

const headList = <T>(list: List<T>): Maybe<T> => list.tag === 'Cons' ? { tag: 'Just', value: list.head } : { tag: 'Nothing' };
const tailList = <T>(list: List<T>): List<T> => list.tag === 'Cons' ? list.tail : Empty;

const print = (value: any): void => {
  // Seseragi型の場合は美しく整形
  if (value && typeof value === 'object' && (
    value.tag === 'Just' || value.tag === 'Nothing' ||
    value.tag === 'Left' || value.tag === 'Right' ||
    value.tag === 'Cons' || value.tag === 'Empty'
  )) {
    console.log(toString(value))
  } 
  // 通常のオブジェクトはそのまま
  else {
    console.log(value)
  }
};
const putStrLn = (value: string): void => console.log(value);
const toString = (value: any): string => {
  // Maybe型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Just') {
    return `Just(${toString(value.value)})`
  }
  if (value && typeof value === 'object' && value.tag === 'Nothing') {
    return 'Nothing'
  }
  
  // Either型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Left') {
    return `Left(${toString(value.value)})`
  }
  if (value && typeof value === 'object' && value.tag === 'Right') {
    return `Right(${toString(value.value)})`
  }
  
  // List型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Empty') {
    return "`[]"
  }
  if (value && typeof value === 'object' && value.tag === 'Cons') {
    const items = []
    let current = value
    while (current.tag === 'Cons') {
      items.push(toString(current.head))
      current = current.tail
    }
    return "`[" + items.join(', ') + "]"
  }
  
  // Tuple型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Tuple') {
    return `(${value.elements.map(toString).join(', ')})`
  }
  
  // 配列の表示
  if (Array.isArray(value)) {
    return `[${value.map(toString).join(', ')}]`
  }
  
  // プリミティブ型
  if (typeof value === 'string') {
    return `"${value}"`
  }
  if (typeof value === 'number') {
    return String(value)
  }
  if (typeof value === 'boolean') {
    return value ? 'True' : 'False'
  }
  
  // 普通のオブジェクト（構造体など）
  if (typeof value === 'object' && value !== null) {
    const pairs = []
    for (const key in value) {
      if (value.hasOwnProperty(key)) {
        pairs.push(`${key}: ${toString(value[key])}`)
      }
    }
    
    // 構造体名を取得（constructor.nameを使用）
    const structName = value.constructor && value.constructor.name !== 'Object' 
      ? value.constructor.name 
      : ''
    
    // 複数フィールドがある場合はインデント表示
    if (pairs.length > 2) {
      return `${structName} {\n  ${pairs.join(',\n  ')}\n}`
    } else {
      return `${structName} { ${pairs.join(', ')} }`
    }
  }
  
  return String(value)
};
const toInt = (value: any): number => {
  if (typeof value === 'number') {
    return Math.trunc(value)
  }
  if (typeof value === 'string') {
    const n = parseInt(value, 10)
    if (isNaN(n)) {
      throw new Error(`Cannot convert "${value}" to Int`)
    }
    return n
  }
  throw new Error(`Cannot convert ${typeof value} to Int`)
};
const toFloat = (value: any): number => {
  if (typeof value === 'number') {
    return value
  }
  if (typeof value === 'string') {
    const n = parseFloat(value)
    if (isNaN(n)) {
      throw new Error(`Cannot convert "${value}" to Float`)
    }
    return n
  }
  throw new Error(`Cannot convert ${typeof value} to Float`)
};
const show = (value: any): void => {
  console.log(toString(value))
};

const arrayToList = curry(<T>(arr: T[]): List<T> => {
  let result: List<T> = Empty;
  for (let i = arr.length - 1; i >= 0; i--) {
    result = Cons(arr[i], result);
  }
  return result;
});

const listToArray = curry(<T>(list: List<T>): T[] => {
  const result: T[] = [];
  let current = list;
  while (current.tag === 'Cons') {
    result.push(current.head);
    current = current.tail;
  }
  return result;
});

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
    case '+': return left + right;
    case '-': return left - right;
    case '*': return left * right;
    case '/': return left / right;
    case '%': return left % right;
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
function __ssrg_Product_f4pl4mu_updateStock(self: Product, newStock: number): Product {
  return (() => { const __tmphttf1m = { ...self, stock: newStock }; return Object.assign(Object.create(Product.prototype), __tmphttf1m); })();
}
function __ssrg_Product_f4pl4mu_sell(self: Product, quantity: number): Either<string, Product> {
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
function __ssrg_Product_f4pl4mu_restock(self: Product, quantity: number): Either<string, Product> {
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
function __ssrg_BankAccount_f4pl4mu_deposit(self: BankAccount, amount: number): Either<string, BankAccount> {
  return (() => {
  const matchValue = amount;
  if (true) {
    const a = matchValue;
    if ((a <= 0)) {
      return Left("入金額は正数である必要があります");
    }
  }  if (true) {
    const a = matchValue;
    return Right((() => { const __tmpvf3cni = { ...self, balance: (self.balance + a) }; return Object.assign(Object.create(BankAccount.prototype), __tmpvf3cni); })());
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}
function __ssrg_BankAccount_f4pl4mu_withdraw(self: BankAccount, amount: number): Either<string, BankAccount> {
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
    return Right((() => { const __tmpplvk8l = { ...self, balance: (self.balance - a) }; return Object.assign(Object.create(BankAccount.prototype), __tmpplvk8l); })());
  } else {
    throw new Error('Non-exhaustive pattern match');
  }
})();
}
function __ssrg_BankAccount_f4pl4mu_getStatement(self: BankAccount): string {
  return (() => {
  const { id, owner, balance } = self;
  return `口座ID: ${id}, 所有者: ${owner}, 残高: ${balance}円`;
})();
}

// Student implementation
function __ssrg_Student_f4pl4mu_addGrade(self: Student, grade: Grade): Student {
  return (() => {
  const grades = Cons(grade, self.grades);
  return (() => { const __tmpcdzt7l = { ...self, grades: grades }; return Object.assign(Object.create(Student.prototype), __tmpcdzt7l); })();
})();
}
function __ssrg_Student_f4pl4mu_getAverageScore(self: Student): Maybe<number> {
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

console.log("=== 実践的な例 ===");

console.log("--- FizzBuzz ---");

const fizzBuzz = (n: number): string => (() => {
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

Array.from({length: 100 - 1 + 1}, (_, i) => i + 1).map(n => console.log(fizzBuzz(n)));

console.log("--- 計算器 ---");

type Operation = { type: 'Add' } | { type: 'Subtract' } | { type: 'Multiply' } | { type: 'Divide' };

const Add = { type: 'Add' as const };
const Subtract = { type: 'Subtract' as const };
const Multiply = { type: 'Multiply' as const };
const Divide = { type: 'Divide' as const };

const calculate = curry((op: Operation, x: number, y: number): Either<string, number> => (() => {
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
})());

const result1 = calculate(Add)(10)(5);

const result2 = calculate(Divide)(20)(4);

const result3 = calculate(Divide)(10)(0);

show(result1);

show(result2);

show(result3);

console.log("--- 在庫管理システム ---");

const apple = new Product({ id: 1, name: "りんご", price: 100, stock: 50 });

show(apple);

const soldApple = __dispatchMethod(apple, "sell", 10);

show(soldApple);

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

show(restockedApple);

console.log("--- 銀行システム ---");

type Transaction = { type: 'Deposit', data: Array<number> } | { type: 'Withdraw', data: Array<number> } | { type: 'Transfer', data: Array<number | number> };

const Deposit = (data0: number) => ({ type: 'Deposit' as const, data: [data0] });
const Withdraw = (data0: number) => ({ type: 'Withdraw' as const, data: [data0] });
const Transfer = (data0: number, data1: number) => ({ type: 'Transfer' as const, data: [data0, data1] });

const account = new BankAccount({ id: 1001, owner: "田中太郎", balance: 50000 });

show(account);

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

show(step1);

show(step2);

show(step3);

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

show(statement);

const failedWithdraw = __dispatchMethod(account, "withdraw", 100000);

show(failedWithdraw);

const invalidDeposit = __dispatchMethod(account, "deposit", -1000);

show(invalidDeposit);

console.log("--- 成績管理システム ---");

const calculateAverage = curry((grades: List<Grade>, sum: number, count: number): number => (() => {
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
})());

const mathGrade = new Grade({ subject: "数学", score: 80 });

const englishGrade = new Grade({ subject: "英語", score: 90 });

const student = new Student({ name: "佐藤花子", grades: Cons(mathGrade, Empty) });

show(student);

const student_prime = __dispatchMethod(student, "addGrade", englishGrade);

show(student_prime);

const average = __dispatchMethod(student_prime, "getAverageScore");

show(average);

console.log("--- 実践的なプログラミングの原則 ---");

console.log("実践的なプログラムでは、型安全性とエラーハンドリングが重要です");

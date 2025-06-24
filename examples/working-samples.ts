// Generated TypeScript code from Seseragi

// Seseragi runtime helpers

// カリー化関数のヘルパー
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

// パイプライン演算子のヘルパー
const pipe = <T, U>(value: T, fn: (arg: T) => U): U => fn(value);

// 逆パイプ演算子のヘルパー
const reversePipe = <T, U>(fn: (arg: T) => U, value: T): U => fn(value);

// モナドバインドのヘルパー（Maybe用）
const bind = <T, U>(maybe: T | null | undefined, fn: (value: T) => U | null | undefined): U | null | undefined => {
  return maybe != null ? fn(maybe) : null;
};

// 畳み込みモノイドのヘルパー
const foldMonoid = <T>(arr: T[], empty: T, combine: (a: T, b: T) => T): T => {
  return arr.reduce(combine, empty);
};

console.log("=== Working Seseragi Samples ===");

console.log("--- Basic Values and Output ---");

const name: string = "Seseragi";

const version: number = 1;

const isReady: boolean = true;

console.log(name);

console.log(String(version));

console.log(String(isReady));

console.log("--- Basic Functions ---");

const greet = (name: string): string => ("Hello, " + name);

const square = (x: number): number => (x * x);

const double = (x: number): number => (x * 2);

console.log(greet("World"));

console.log(String(square(5)));

console.log(String(double(7)));

console.log("--- Curried Functions ---");

const add = curry((x: number, y: number): number => (x + y));

const multiply = curry((x: number, y: number): number => (x * y));

console.log(String(add(10)(5)));

console.log(String(multiply(6)(7)));

const add10 = add(10);

const multiplyBy3 = multiply(3);

console.log(String(add10(8)));

console.log(String(multiplyBy3(9)));

console.log("--- Conditionals ---");

const max = curry((x: number, y: number): number => ((x > y) ? x : y));

const isEven = (x: number): boolean => (((x % 2) === 0) ? true : false);

console.log(String(max(15)(23)));

console.log(String(isEven(8)));

console.log(String(isEven(7)));

console.log("--- String Operations ---");

const fullName = curry((first: string, last: string): string => ((first + " ") + last));

const makeGreeting = (name: string): string => (("Welcome, " + name) + "!");

console.log(fullName("John")("Doe"));

console.log(makeGreeting("Alice"));

console.log("--- Recursive Functions ---");

const factorial = (n: number): number => ((n <= 1) ? 1 : (n * factorial((n - 1))));

const power = curry((base: number, exp: number): number => ((exp === 0) ? 1 : (base * power(base)((exp - 1)))));

console.log(String(factorial(5)));

console.log(String(power(2)(3)));

console.log(String(power(3)(2)));

console.log("--- Functional Style Calls ---");

console.log("Functional style works!");

console.log("This adds a newline");

const number: number = 42;

console.log(String(number));

console.log(("Result: " + String(add(20)(22))));

console.log("=== All samples completed successfully! ===");

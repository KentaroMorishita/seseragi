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

console.log("Hello, Seseragi!");

const name: string = "World";

console.log(name);

const age: number = 25;

console.log(String(age));

const result: number = (10 + 20);

console.log(String(result));

const greet = (name: string): void => console.log(("Hello, " + name));

console.log("Name:", name, "Age:", String(age));

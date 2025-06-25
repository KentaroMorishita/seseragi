// Generated TypeScript code from Seseragi

// Seseragi minimal runtime

type Maybe<T> = { tag: 'Just'; value: T } | { tag: 'Nothing' };
type Either<L, R> = { tag: 'Left'; value: L } | { tag: 'Right'; value: R };

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

const Just = <T>(value: T): Maybe<T> => ({ tag: 'Just', value });
const Nothing: Maybe<never> = { tag: 'Nothing' };

const Left = <L>(value: L): Either<L, never> => ({ tag: 'Left', value });
const Right = <R>(value: R): Either<never, R> => ({ tag: 'Right', value });

const print = (value: any): void => console.log(value);

console.log("=== Seseragi Tutorial ===");

console.log("--- Basic Types ---");

const intValue = 42;

const floatValue = 3.14;

const stringValue = "Hello, Seseragi!";

const boolValue = true;

console.log(intValue);

console.log(floatValue);

console.log(stringValue);

console.log(boolValue);

console.log("--- Functions ---");

const getMessage = (): string => "Hello from function!";

const getNumber = (): number => 42;

const add = curry((x: number, y: number): number => (x + y));

const double = (x: number): number => (x * 2);

const sum = add(10)(5);

const doubled = double(7);

const message = getMessage;

const number = getNumber;

console.log(sum);

console.log(doubled);

console.log(message);

console.log(number);

console.log("--- Block Functions ---");

const processNumber = (x: number): number => (() => {
  const doubled = (x * 2);
  const incremented = (doubled + 1);
  return incremented;
})();

const formatMessage = (x: number): string => (() => {
  return "Result";
})();

const complexCalculation = curry((a: number, b: number): number => (() => {
  const sum = (a + b);
  const product = (a * b);
  return ((sum > product) ? sum : product);
})());

const processed = processNumber(5);

const formatted = formatMessage(42);

const calculated = complexCalculation(4)(6);

console.log(processed);

console.log(formatted);

console.log(calculated);

console.log("--- Conditional ---");

const max = curry((x: number, y: number): number => ((x > y) ? x : y));

const maximum = max(10)(5);

console.log(maximum);

console.log("--- Maybe Type ---");

const someValue = Just(42);

const nothingValue = Nothing;

console.log(someValue);

console.log(nothingValue);

const safeDivide = curry((x: number, y: number): Maybe<number> => ((y === 0) ? Nothing : Just((x / y))));

const result1 = safeDivide(10)(2);

const result2 = safeDivide(10)(0);

console.log(result1);

console.log(result2);

console.log("--- Either Type ---");

const successValue = Right(42);

const errorValue = Left("Error occurred");

console.log(successValue);

console.log(errorValue);

const parseNumber = (str: string): Either<string, number> => ((str === "42") ? Right(42) : Left("Invalid number"));

const parsed1 = parseNumber("42");

const parsed2 = parseNumber("invalid");

console.log(parsed1);

console.log(parsed2);

console.log("Tutorial completed!");

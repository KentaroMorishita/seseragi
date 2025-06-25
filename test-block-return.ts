// Generated TypeScript code from Seseragi

// Seseragi minimal runtime

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

const apply = <T, U>(fn: (arg: T) => U, value: T): U => fn(value);

const print = (value: any): void => console.log(value);

const processNumber = (x: number): number => (() => {
  const doubled = (x * 2);
  const incremented = (doubled + 1);
  return incremented;
})();

const conditionalReturn = (x: number): number => (() => {
  return ((x > 10) ? (x * 2) : (x + 5));
})();

const complexCalculation = curry((a: number, b: number): number => (() => {
  const sum = (a + b);
  const product = (a * b);
  return ((sum > product) ? sum : product);
})());

console.log("=== Block Functions Test ===");

console.log(processNumber(5));

console.log(conditionalReturn(15));

console.log(conditionalReturn(3));

console.log(complexCalculation(4)(6));

# Seseragi Learning Guide

---

## Part 1: First Steps with Seseragi

This chapter introduces you to the Seseragi language, what problems it solves, and guides you through running your first program and understanding its fundamental concepts.

### 1.1. Introduction

Seseragi is a new statically typed language that compiles to TypeScript.

Built on TypeScript's powerful type system and vast ecosystem, it aims to enable safer and more expressive code writing.

> **About the name "Seseragi"**
>
> Seseragi is named after the Japanese word "せせらぎ" (a babbling brook). Like the smooth flow of a stream, it aims to be a language with natural, flowing syntax that doesn't impede your thinking. The name also reflects the developer's Japanese heritage and desire to preserve sounds from the Japanese language.

#### Seseragi for TypeScript Developers

If you're familiar with TypeScript, learning Seseragi will be smooth. Seseragi aims to solve several challenges that TypeScript developers face daily through language features.

- **Freedom from `null` and `undefined`**
  By introducing special types called `Maybe` and `Either`, it completely prevents runtime errors caused by `null` or `undefined` at compile time. This frees you from cumbersome null checks that appear throughout your code.

- **Simplification of complex conditional logic**
  With powerful `match` expressions (pattern matching), you can replace nested `if` statements and complex `switch` statements with readable, comprehensive, and safe code.

- **Predictable state management**
  Data is **immutable** by default. Similar to using `const` consistently in TypeScript, but this is the language standard. This prevents unintended side effects and dramatically simplifies application state management.

- **More precise type expression**
  Introduces algebraic data types (ADTs) to express business logic more accurately and flexibly through types.

**Important**: Seseragi is not a pure functional language. While it incorporates practical ideas from functional languages, it aims to be a familiar language on the TypeScript continuum.

---

### 1.2. Environment Setup and First Program

Let's run Seseragi for the first time.

#### Installation

Install the project dependencies. (This project uses `bun`)

```bash
bun install
```

#### Hello, World!

1. First, create a file named `hello.ssrg`.

2. Write the following code in the file:

    ```rust
    // hello.ssrg
    print "Hello, Seseragi!"
    ```

3. Run it with the `seseragi run` command:

    ```bash
    seseragi run hello.ssrg
    ```

4. If `Hello, Seseragi!` appears in the console, you've succeeded.

#### CLI Tool Overview

Seseragi provides three main commands:

- `seseragi run <file>`
  Directly executes a file. Internally, compilation to TypeScript and execution happen simultaneously.

- `seseragi compile <file>` (or `seseragi <file>`)
  Converts Seseragi code to TypeScript files (`.ts`).

- `seseragi fmt <file>`
  Formats code according to official formatting rules.

---

### 1.3. Fundamental Concepts of Seseragi

Before you start writing Seseragi code, it's helpful to understand two important concepts that differ from many other languages.

#### An Expression-Centered World

Many languages distinguish between "statements" (which don't return values) and "expressions" (which return values).

```typescript
// TypeScript's `if` is a "statement" and doesn't return a value
let message: string;
if (score > 80) {
  message = "Pass";
} else {
  message = "Fail";
}
```

In contrast, Seseragi is designed so that **most constructs are "expressions" that return values**.

This makes code more concise and helps avoid variable reassignment.

```rust
// Seseragi's `if` is an "expression" that returns a value
let message = if score > 80 then "Pass" else "Fail"
```

This "everything returns a value" philosophy is an important principle underlying many of Seseragi's features.

#### The Importance of Immutability

In Seseragi, all variables bound with `let` are **immutable**.

Once a value is set, it cannot be changed.

This is similar to always using `const` in TypeScript and avoiding reassignment with `let`.

In Seseragi, this safe practice is the language standard.

Why is immutability important?

- **Predictability**: Since values don't change midway through, program behavior is easier to follow.
- **Side effect suppression**: Prevents bugs where "values were changed without my knowledge."
- **Safe concurrent processing**: Multiple places can reference the same data without race conditions (advanced topic).

When you want to change part of data, instead of modifying the original data, you create a copy with only the changed parts updated.

Seseragi provides syntax to do this efficiently.

---

## Part 2: Basic Programming Elements

This chapter covers the basic building blocks of programming—"variables," "types," "operators," "functions," and "control flow"—with Seseragi's unique approach.

### 2.1. Variables, Basic Types, and Operators

#### Variable Binding and Basic Types

Use `let` to bind values to names (variables).

Since Seseragi has type inference, you can omit type annotations.

```rust
let companyName = "TechCorp" // String type
let yearEstablished = 2024     // Int type
let rating = 4.5               // Float type
let isPublic = True            // Bool type
```

When you want to specify types explicitly, add `: TypeName` after the variable name.

```rust
let companyName: String = "TechCorp"
let yearEstablished: Int = 2024
let rating: Float = 4.5
let isPublic: Bool = True
```

#### String Interpolation

In strings surrounded by backticks (`` ` ``), you can embed variables using `${...}`.

```rust
let name = "Alice"
let message = `Hello, ${name}!`
```

#### Basic Operators

Standard arithmetic operators (`+`, `-`, `*`, `/`, `%`), exponentiation operator (`**`), comparison operators (`==`, `!=`, `>`, `<`), and logical operators (`&&`, `||`, `!`) are available.

---

### 2.2. Functions: The Heart of Seseragi

In Seseragi, functions are not just collections of procedures but the most important "components" for assembling programs.

#### Definition with `fn`

Define functions using the `fn` keyword.

Use `=` when the body is a single expression, or `{}` blocks when describing multiple processes.

```rust
// Single-expression function using `=`
fn square x: Int -> Int = x * x

// Block syntax function using `{}`
// The last expression in the block becomes the return value
fn getHypotenuse a: Float -> b: Float -> Float {
  let a2 = a ** 2.0
  let b2 = b ** 2.0
  (a2 + b2) ** 0.5
}
```

#### Automatic Currying and Partial Application

All functions in Seseragi are automatically **curried**.

This enables **partial application**, where you can pass only some arguments to a function to easily create new functions specialized for specific tasks.

```rust
fn add x: Int -> y: Int -> Int = x + y

// Partial application: create a function that "adds 10"
let addTen = add 10

show $ addTen 3 // 13
```

#### Higher-Order Functions and Lambda Expressions

Functions can be treated as values (**higher-order functions**). You can define lambda expressions (anonymous functions) with `\`.

```rust
// Double each element of an array (operators detailed in later chapters)
let numbers = [1, 2, 3]
let doubled = (\x -> x * 2) <$> numbers
// doubled is [2, 4, 6]
```

---

### 2.3. Control Flow: Mastering Branching

#### `if-then-else` Expressions and Ternary Operator

Seseragi's `if` is an **expression** that returns a value, and `else` cannot be omitted.

```rust
let score = 75
let grade = if score >= 80 then "A" else if score >= 60 then "B" else "C"
```

As more concise syntactic sugar, you can also use the ternary operator (`? :`).

```rust
let grade = score >= 80 ? "A" : score >= 60 ? "B" : "C"
```

#### Introduction to Pattern Matching (`match`)

The `match` expression is a more safe and expressive replacement for chains of `if-else` or complex `switch` statements.

It checks which "pattern" a value matches and executes the corresponding process.

The compiler checks whether all possibilities are covered, helping prevent bugs.

```rust
fn getHttpStatusMessage code: Int -> String = match code {
  200 -> "OK"
  404 -> "Not Found"
  500 -> "Internal Server Error"
  _ -> "Unknown Status" // `_` is the default case when no pattern matches
}

show $ getHttpStatusMessage 404 // "Not Found"
```

This chapter introduced basic literal (concrete value) matching.

The true power of `match` is unleashed when combined with complex data structures, which we'll learn in Part 3 and beyond.

---

## Part 3: Mastering Data Structures

This chapter covers various "containers" for organizing data and efficient ways to work with them.

### 3.1. Basic Composite Data

#### Tuples: Temporary Value Pairing

Tuples are the simplest way to temporarily group multiple values together.

They're particularly useful when you want to define multiple values with `let` binding or evaluate multiple values at once with `match` expressions.

```rust
let myTuple = (10, "hello", True)

// Destructuring assignment to extract tuple elements
let (num, str, bool) = myTuple
// num is 10, str is "hello", bool is True

// Using with pattern matching
match myTuple {
  (10, _, True) -> print "Match!"
  _ -> print "No match"
}
```

Tuples are temporary structures for immediate use; records or structs (explained next) are typically used for long-term data.

#### Records: Data with Named Fields

Records are collections of name-value pairs, similar to TypeScript's object literals or interfaces.

Since data is accessed by field names, the intent is clearer than with tuples.

```rust
let user = { name: "Alice", age: 30, active: True }

// Field access
show user.name // "Alice"

// Records follow "structural typing"
// Fields with the same structure are treated as the same type
// (type aliases detailed in Part 4)
fn greet u: { name: String } -> String {
  `Hello, ${u.name}!`
}

show $ greet user // "Hello, Alice!"
```

---

### 3.2. Collection Choice and Usage: `Array` vs `List`

Seseragi provides two types of collections for grouping multiple elements: `Array` and `List`.

While similar, they have different strengths and weaknesses.

#### `Array`: TypeScript-like Fast Access

`Array` can be used much like TypeScript arrays.

Internally compiled to JavaScript arrays, making integration with existing JavaScript libraries easy.

- **When to use**: When frequent random access by index is needed or when checking element count frequently.
- **Definition**: `[1, 2, 3, 4, 5]`
- **Safe access**: `myArray[index]` returns `Maybe` type to prevent out-of-bounds errors.

```rust
let numbers = [10, 20, 30]

show numbers.length // 3

match numbers[0] {
  Just n -> show `First element: ${n}` // First element: 10
  Nothing -> show "No elements"
}
```

#### `List`: Immutable and Recursive Data Processing

`List` is a "linked list" traditionally used in functional programming.

Its essence is a recursive structure that is either "**`Empty` (empty list)**" or "**`Cons h t` (head element `h` and remaining list `t`)**".

- **When to use**: For recursive algorithms that process lists from the head sequentially, or when decomposing lists with pattern matching.

- **Definition (step-by-step explanation)**:

    1. **Basic structure**: `List` is composed of `Cons` and `Empty`.
        ```rust
        let list1 = Cons 1 (Cons 2 (Cons 3 Empty))
        ```

    2. **`:` operator**: Syntactic sugar for `Cons`. `:` is right-associative, so you can chain without parentheses.
    Also, `` `[] `` is syntactic sugar for `Empty`.
        ```rust
        let list2 = 1 : 2 : 3 : `[] // equivalent to list1
        ```

    3. **`` `[...] `` syntax**: The most intuitive and recommended syntactic sugar.
        ```rust
        let list3 = `[1, 2, 3] // equivalent to list1, list2
        ```

- **Basic operations**: Use `^` (head) and `>>` (tail) operators.

```rust
let numbers = `[10, 20, 30]

show $ ^numbers // Just 10
show $ >>numbers // `[20, 30]

// Powerful combination with pattern matching
fn sum list: List<Int> -> Int = match list {
  `[] -> 0 // 0 for empty list
  `[h, ...t] -> h + sum t // decompose into head(h) and tail(t), recursively sum
}

show $ sum numbers // 60
```

---

### 3.3. Declarative Collection Operations: Comprehensions

Instead of `for` loops, Seseragi uses **comprehensions** to declaratively generate new collections from existing ones.

This feature allows you to write processes similar to combining TypeScript's `map` and `filter` with more intuitive syntax.

```rust
// Create an array of squares of even numbers from 1 to 10
let evenSquares = [x * x | x <- 1..=10, x % 2 == 0]
// evenSquares is [4, 16, 36, 64, 100]
```

- `x <- 1..=10`: Extract each element from the `1..=10` collection as `x`. (Generator)
- `x % 2 == 0`: Target only `x` that are even. (Guard)
- `x * x`: Square the targeted `x`. (Output expression)

Comprehensions work with both `Array` (`[]`) and `List` (`` `[] ``).

---

## Part 4: Powerful Type System

This chapter delves deep into Seseragi's type definition features that support its expressiveness.

If you're familiar with TypeScript's type system, you'll gain a deeper understanding of the differences and advantages.

### 4.1. Defining Your Own Types

#### Type Aliases (`type`): Giving Types Alternative Names

The `type` keyword is used to give meaningful alternative names to existing types.

This is exactly the same as TypeScript's `type` aliases.

It doesn't create new types but merely provides aliases.

```rust
// Give alternative names to basic types
type UserId = Int
type Email = String

// Give alternative names to complex types
type UserProfile = {
    name: String,
    age: Int
}
type OnSuccess = (data: String) -> Unit
```

Using type aliases clarifies code intent and serves as documentation.

#### Structs (`struct`): Encapsulating Data and Behavior

`struct` defines a "structure" that groups related data together. This is similar to TypeScript's `class` but with a more data-centric design.

An important feature of `struct` is that it uses **nominal typing**.

This means that even if the field structure is identical, structs with different names are treated as completely different types.

This prevents unintended type confusion.

```rust
struct Point {
    x: Float,
    y: Float
}
struct Vector {
    x: Float,
    y: Float
}

let p = Point { x: 1.0, y: 2.0 }
let v = Vector { x: 1.0, y: 2.0 }

// p and v are incompatible even though they have the same shape, because they're different types
```

#### `impl`: Methods and Operator Overloading

Using `impl` blocks, you can add related behaviors (methods) and custom operator implementations (overloading) to defined `struct`s.

```rust
impl Point {
  // `self` refers to the instance itself. Equivalent to `this` in TypeScript.
  fn distanceToOrigin self -> Float {
    (self.x ** 2.0 + self.y ** 2.0) ** 0.5
  }

  // Define Point-specific behavior for the `+` operator
  operator + self -> other -> Point {
    Point { x: self.x + other.x, y: self.y + other.y }
  }
}

let p1 = Point { x: 3.0, y: 4.0 }
let p2 = Point { x: 1.0, y: 2.0 }

// Method call
show $ p1 distanceToOrigin() // 5.0

// Using overloaded operator
show $ p1 + p2 // Point { x: 4.0, y: 6.0 }
```

---

### 4.2. Algebraic Data Types (ADT): Precisely Expressing State

Algebraic Data Types (ADTs) are one of the most powerful features of Seseragi's type system. They allow you to strictly express at the type level that "**this type takes exactly one of these predetermined forms**".

This is similar to TypeScript's union types, but each variation is clearly distinguished by "tags," making it safer and having excellent compatibility with pattern matching.

#### ADT Definition

Define using the `type` keyword and `|`.

Each variation (type constructor) must start with a capital letter.

```rust
// Simplest ADT (enum type)
type Status =
  | Idle
  | Loading
  | Success
  | Failure

// ADT where each variation holds data
type WebEvent =
  | PageLoad
  | Click { x: Int, y: Int }
  | KeyPress String
```

#### ADT Advantages: Eliminating Impossible States

For example, consider API request states.

In TypeScript, it's common to manage this with multiple states like `isLoading`, `data`, `error`:

```typescript
interface ApiState {
  isLoading: boolean;
  data?: string;      // specific type
  error?: Error;
}
```

This design allows for impossible states like "`isLoading` is `true` but `data` also exists" or "`data` and `error` exist simultaneously".

Using ADTs, you can define these states as mutually exclusive variations, completely eliminating impossible states at the type system level.

```rust
type ApiState =
  | Loading
  | Success String
  | Failure String

let state: ApiState = Loading

// Pattern matching allows safe and comprehensive handling of each state
fn render state: ApiState -> String = match state {
  Loading -> "Loading..."
  Success data -> `Data: ${data}`
  Failure msg -> `Error: ${msg}`
}
```

---

## Part 5: Safe Error Handling

Seseragi provides a type system-based safe error handling mechanism that differs from the common `null`, `undefined`, and `try-catch` exception handling in many languages.

### 5.1. Why are `null` and `try-catch` Problematic?

`null` and `undefined` in TypeScript (and JavaScript) are convenient mechanisms for indicating "no value," but they're also the biggest cause of runtime errors like `TypeError: Cannot read properties of null`, known as "null pointer exceptions."

Code becomes cluttered with checks like `if (value != null)` everywhere, and bugs from missed checks are endless.

Also, `try-catch` exception handling tends to separate error-generating code from error-handling code.

You can't tell from function signatures (types) which functions might throw exceptions, leading to programming while fearing invisible errors.

Seseragi solves these problems by "**expressing the possibility of failure through types**".

---

### 5.2. `Maybe<T>` Type: When a Value Might Not Exist

`Maybe<T>` is a type indicating that a value is either "**exists (`Just T`)**" or "**doesn't exist (`Nothing`)**".

This is used when function return values are optional or in situations where results might not be found, like array or dictionary searches.

Using functions that return `Maybe` types eliminates the possibility of forgetting `null` checks.

This is because you cannot directly extract contents from `Maybe` type values—**you must use `match` expressions to handle both `Just` and `Nothing` cases**.

The compiler enforces this check.

```rust
let names = ["Alice", "Bob", "Charlie"]

// `[]` access returns Maybe<String>
let first = names[0]
let fifth = names[4]

fn showName maybeName: Maybe<String> -> String = match maybeName {
  Just name -> `Name: ${name}`
  Nothing -> "No name"
}

show $ showName first // Name: Alice
show $ showName fifth // No name
```

---

### 5.3. `Either<L, R>` Type: When You Want to Convey "Reason for Failure"

`Either<L, R>` is a more powerful version of `Maybe`.

It indicates whether a process "**succeeded (`Right R`)**" or "**failed (`Left L`)**".

The biggest difference from `Maybe` is that when it fails, it can **hold error information `L` explaining why it failed**.

```rust
// Function to parse string to number
fn parseInt text: String -> Either<String, Int> = match text {
  "123" -> Right 123
  "abc" -> Left "Invalid number format"
  _ -> Right 0 // for simplicity
}

let result1 = parseInt "123"
let result2 = parseInt "abc"

match result2 {
  Right num -> show `Parse success: ${num}`
  Left err -> show `Parse failed: ${err}` // Parse failed: Invalid number format
}
```

The advantage of using `Either` over `try-catch` is that **errors are explicitly stated as part of the function's return type**.

This makes it obvious that the function can fail, forcing the caller to handle error cases.

---

### 5.4. Safe Process Chaining with `>>=`

The true value of `Maybe` and `Either` lies in their ability to safely **chain (pipeline)** processes that might fail.

If you want to perform multiple processes in sequence and stop processing if any one fails, TypeScript requires nested `if` statements or `try-catch` blocks.

In Seseragi, you can write this flow remarkably cleanly using the `>>=` (bind) operator.

`m >>= f` executes the next process by passing the contents to function `f` only if `m` succeeds (`Just` or `Right`).

If `m` fails (`Nothing` or `Left`), `f` is not executed, and the failure becomes the final result.

#### Concrete Example: Safe Configuration Value Retrieval

Consider a series of processes: finding configuration by user ID, then getting the hostname from that configuration.

```typescript
// Nested null checks in TypeScript
interface Config { host: string; port: number; }

function findConfig(userId: string): Config | undefined { ... }
function getHost(config: Config): string | undefined { ... }

const config = findConfig("user1");
if (config) {
  const host = getHost(config);
  if (host) {
    console.log(host); // "example.com"
  } else {
    console.log("Hostname not found");
  }
} else {
  console.log("Configuration not found");
}
```

The same process in Seseragi:

```rust
type Config = { host: String, port: Int }

// Get configuration from user ID. User might not exist.
fn findConfig userId: String -> Maybe<Config> = match userId {
  "user1" -> Just $ { host: "example.com", port: 8080 } as Config
  _ -> Nothing
}

// Get hostname from Config
fn getHost config: Config -> Maybe<String> = Just config.host

// Process chaining
let hostname = Just "user1"
  >>= findConfig
  >>= getHost

show hostname // Just "example.com"

// When user doesn't exist
let hostname2 = Just "user2"
  >>= findConfig
  >>= getHost

show hostname2 // Nothing
```

You can see that nested `if` statements are replaced with a straight line of processing flow using `>>=` operators.

Since `>>=` automatically handles intermediate `Nothing` values, developers can focus only on connecting successful case processes (`findConfig` and `getHost`).

---

## Part 6: Advanced Topics

You've now learned all the basic elements of Seseragi in the previous chapters.

This final chapter explains advanced techniques useful for more practical coding by combining that knowledge, and discusses future prospects.

### 6.1. Pattern Matching Deep Dive

Beyond the basic usage we've seen, `match` expressions support various patterns for concisely and safely decomposing complex data structures.

#### Guards with `when`

You can execute a process only when it matches a pattern and also satisfies additional conditions (guards).

```rust
fn checkNumber n: Int -> String = match n {
  0 -> "Zero"
  x when x % 2 == 0 -> `Even: ${x}`
  x when x % 2 != 0 -> `Odd: ${x}`
  _ -> "?"
}

show $ checkNumber 0 // "Zero"
show $ checkNumber 10 // "Even: 10"
show $ checkNumber 7  // "Odd: 7"
```

#### Data Structure Decomposition

The true power of `match` expressions is unleashed when combined with complex data structures like `List` and `struct`.

You can dive deep into the structure's internals and directly extract elements that match specific patterns.

- **List decomposition**: The `h : t` pattern can decompose a list into head element (`h`) and remaining list (`t`).

    ```rust
    fn describeList list: List<Int> -> String = match list {
      `[] -> "Empty list"
      `[h] -> `List with one element: ${h}`
      `[h, ...t] -> `Head is ${h}, and there are more elements`
    }
    ```

- **Array decomposition**: You can match fixed-length array patterns within `[]`.

    ```rust
    fn describeArray arr: Array<Int> -> String = match arr {
      [] -> "Empty array"
      [a] -> `One element: ${a}`
      [a, b] -> `Two elements: ${a} and ${b}`
      _ -> "Three or more elements"
    }
    ```

---

### 6.2. TypeScript Integration (Future Prospects)

Since Seseragi compiles to TypeScript, it can theoretically integrate with the vast JavaScript/TypeScript ecosystem.

#### Generated Code

The `.ts` files generated by the `seseragi compile` command are designed to be readable, straightforward code for humans.

For example, Seseragi's `struct` converts to TypeScript's `class`, and ADTs convert to tagged union types, making them relatively easy to use from the TypeScript side.

#### FFI (Foreign Function Interface)

In the future, introduction of a Foreign Function Interface (FFI) for type-safely calling existing TypeScript functions and libraries from within Seseragi code is being considered.

This would allow using functionality from packages installed via `npm` while combining them with Seseragi's powerful type system, greatly expanding the language's possibilities.

---

This concludes the Seseragi Learning Guide. Great work!

We hope this guide helps make your Seseragi programming experience more enjoyable and productive.
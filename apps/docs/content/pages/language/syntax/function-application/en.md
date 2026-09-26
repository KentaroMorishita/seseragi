---json
{
  "schema": 1,
  "id": "language.syntax.function-application",
  "locale": "en",
  "route": "/docs/language/syntax/function-application/",
  "kind": "language",
  "title": "Function application",
  "summary": "Apply curried functions with whitespace and understand binding, evaluation order, type arguments, and partial application.",
  "availability": "available",
  "spec": [
    "docs/spec/01-syntax.md#1.4",
    "docs/spec/02-types.md#2.7",
    "docs/spec/03-data-and-expressions.md#3.1"
  ],
  "examples": ["examples/spec/lessons/02-values-and-functions.ssrg"],
  "prerequisites": [],
  "related": [],
  "next": "",
  "referenceIdentities": []
}
---
# Function application

Seseragi applies a function by writing an argument after it. Application binds more tightly than every infix operator and associates to the left, so `add 1 2` means `(add 1) 2`.

## Apply one argument at a time

A function declaration uses `fn`. Each parameter arrow introduces one curried parameter, and each application supplies one argument. Applying only the first argument returns another function.

:::example {"title":"Curried functions and left-associative application","source":"examples/spec/lessons/02-values-and-functions.ssrg"}
:::

The parameters bind from left to right. In `add base 22`, Seseragi evaluates `add`, then `base`, then `22`, exactly once each. Ordinary application is strict: after the final argument is present, the function body runs immediately.

:::common-mistake {"title":"Parentheses do not create a multi-argument call"}
`f(x, y)` is not Seseragi call syntax. Parentheses group one expression, construct a tuple, or spell Unit. Supply curried arguments with `f x y`; pass a tuple only when the function's parameter type is a tuple.
:::

## Partial application

If `add` has type `Int -> Int -> Int`, then `add 20` has type `Int -> Int`. The returned function captures the first argument without evaluating the function body early. The body runs when the remaining argument is supplied.

Functions declared without an explicit parameter receive an anonymous Unit parameter. Their type is `Unit -> A`, and callers must apply `()` explicitly.

## Type arguments and operators

Explicit type arguments attach directly to the callee: `identity<String> value`. Whitespace changes the parse, so `identity < value` starts a comparison instead. Custom infix operators also require whitespace around their operands, while an operator value such as `(<+>)` can be applied as an ordinary curried function.

:::design-rationale {"title":"Application does not run an Effect"}
Applying a function that returns `Effect<R, E, A>` constructs an Effect value. It does not execute the Effect body. Execution begins only at an explicit runtime boundary, which keeps evaluation order and external work visible in the type.
:::

## Group complex arguments

Application takes the next tightly bound expression as its argument. Use parentheses when an infix expression, conditional, match, do block, or lambda must be supplied as one argument and the surrounding precedence would otherwise produce a different parse.

A non-function callee is a type error. An application also fails type checking when argument types do not match, explicit type arguments are incomplete, or the remaining trait constraints cannot be resolved.

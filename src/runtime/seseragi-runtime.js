// src/runtime/seseragi-runtime.ts
var curry = (fn) => {
  return function curried(...args) {
    if (args.length >= fn.length) {
      return fn.apply(this, args)
    } else {
      return function (...args2) {
        return curried.apply(this, args.concat(args2))
      }
    }
  }
}
var pipe = (value, fn) => fn(value)
var reversePipe = (fn, value) => fn(value)
var apply = (fn, value) => fn(value)
var Just = (value) => ({ tag: "Just", value })
var Nothing = { tag: "Nothing" }
var mapMaybe = (fa, f) => (fa.tag === "Just" ? Just(f(fa.value)) : Nothing)
var pureMaybe = (value) => Just(value)
var applyMaybe = (ff, fa) =>
  ff.tag === "Just" && fa.tag === "Just" ? Just(ff.value(fa.value)) : Nothing
var bindMaybe = (ma, f) => {
  return ma.tag === "Just" ? f(ma.value) : Nothing
}
var isJust = (maybe) => maybe.tag === "Just"
var isNothing = (maybe) => maybe.tag === "Nothing"
var fromMaybe = (defaultValue, maybe) =>
  maybe.tag === "Just" ? maybe.value : defaultValue
var Left = (value) => ({ tag: "Left", value })
var Right = (value) => ({ tag: "Right", value })
var mapEither = (fa, f) => (fa.tag === "Right" ? Right(f(fa.value)) : fa)
var pureEither = (value) => Right(value)
var applyEither = (ff, fa) =>
  ff.tag === "Right" && fa.tag === "Right"
    ? Right(ff.value(fa.value))
    : ff.tag === "Left"
      ? ff
      : fa
var bindEither = (ea, f) => {
  return ea.tag === "Right" ? f(ea.value) : ea
}
var isLeft = (either) => either.tag === "Left"
var isRight = (either) => either.tag === "Right"
var fromLeft = (defaultValue, either) =>
  either.tag === "Left" ? either.value : defaultValue
var fromRight = (defaultValue, either) =>
  either.tag === "Right" ? either.value : defaultValue
var bind = (maybe, fn) => bindMaybe(maybe, fn)
var foldMonoid = (arr, empty, combine) => {
  return arr.reduce(combine, empty)
}
var print = (value) => console.log(value)
var putStrLn = (value) => console.log(value)
var toString = (value) => String(value)
var identity = (value) => value
var compose = (f, g) => (a) => f(g(a))
export {
  toString,
  reversePipe,
  putStrLn,
  pureMaybe,
  pureEither,
  print,
  pipe,
  mapMaybe,
  mapEither,
  isRight,
  isNothing,
  isLeft,
  isJust,
  identity,
  fromRight,
  fromMaybe,
  fromLeft,
  foldMonoid,
  curry,
  compose,
  bindMaybe,
  bindEither,
  bind,
  applyMaybe,
  applyEither,
  apply,
  Right,
  Nothing,
  Left,
  Just,
}

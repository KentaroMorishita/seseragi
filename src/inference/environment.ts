/**
 * 型環境ユーティリティ (Type Environment) for Seseragi Language
 *
 * 組み込み型・関数の初期環境を定義
 */

import * as AST from "../ast"
import { PolymorphicTypeVariable } from "./type-variables"

/**
 * 組み込み関数・型の初期環境を作成
 * 純粋関数：状態を持たず、常に同じ結果を返す
 */
export function createInitialEnvironment(): Map<string, AST.Type> {
  const env = new Map<string, AST.Type>()

  // 組み込み関数のシグネチャを定義

  // print: 'a -> Unit (多相関数)
  const printType = new AST.FunctionType(
    new PolymorphicTypeVariable("a", 0, 0),
    new AST.PrimitiveType("Unit", 0, 0),
    0,
    0
  )
  env.set("print", printType)

  // putStrLn: 'a -> Unit (多相関数)
  const putStrLnType = new AST.FunctionType(
    new PolymorphicTypeVariable("a", 0, 0),
    new AST.PrimitiveType("Unit", 0, 0),
    0,
    0
  )
  env.set("putStrLn", putStrLnType)

  // toString: 'a -> String (多相関数)
  const toStringType = new AST.FunctionType(
    new PolymorphicTypeVariable("a", 0, 0),
    new AST.PrimitiveType("String", 0, 0),
    0,
    0
  )
  env.set("toString", toStringType)

  // toInt: 'a -> Int (多相関数)
  const toIntType = new AST.FunctionType(
    new PolymorphicTypeVariable("a", 0, 0),
    new AST.PrimitiveType("Int", 0, 0),
    0,
    0
  )
  env.set("toInt", toIntType)

  // toFloat: 'a -> Float (多相関数)
  const toFloatType = new AST.FunctionType(
    new PolymorphicTypeVariable("a", 0, 0),
    new AST.PrimitiveType("Float", 0, 0),
    0,
    0
  )
  env.set("toFloat", toFloatType)

  // show: 'a -> Unit (多相関数)
  const showType = new AST.FunctionType(
    new PolymorphicTypeVariable("a", 0, 0),
    new AST.PrimitiveType("Unit", 0, 0),
    0,
    0
  )
  env.set("show", showType)

  // arrayToList: Array<'a> -> List<'a>
  const aTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const arrayToListType = new AST.FunctionType(
    new AST.GenericType("Array", [aTypeVar], 0, 0),
    new AST.GenericType("List", [aTypeVar], 0, 0),
    0,
    0
  )
  env.set("arrayToList", arrayToListType)

  // listToArray: List<'a> -> Array<'a>
  const bTypeVar = new PolymorphicTypeVariable("b", 0, 0)
  const listToArrayType = new AST.FunctionType(
    new AST.GenericType("List", [bTypeVar], 0, 0),
    new AST.GenericType("Array", [bTypeVar], 0, 0),
    0,
    0
  )
  env.set("listToArray", listToArrayType)

  // head: List<'a> -> Maybe<'a>
  const headTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const headType = new AST.FunctionType(
    new AST.GenericType("List", [headTypeVar], 0, 0),
    new AST.GenericType("Maybe", [headTypeVar], 0, 0),
    0,
    0
  )
  env.set("head", headType)

  // tail: List<'a> -> List<'a>
  const tailTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const tailType = new AST.FunctionType(
    new AST.GenericType("List", [tailTypeVar], 0, 0),
    new AST.GenericType("List", [tailTypeVar], 0, 0),
    0,
    0
  )
  env.set("tail", tailType)

  // List constructors for pattern matching and expressions
  // Empty : List<'a>
  const emptyTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const emptyType = new AST.GenericType("List", [emptyTypeVar], 0, 0)
  env.set("Empty", emptyType)

  // Cons : 'a -> List<'a> -> List<'a>
  const consTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const consListType = new AST.GenericType("List", [consTypeVar], 0, 0)
  const consType = new AST.FunctionType(
    consTypeVar,
    new AST.FunctionType(consListType, consListType, 0, 0),
    0,
    0
  )
  env.set("Cons", consType)

  // Maybe constructors for pattern matching and expressions
  // Nothing : Maybe<'a>
  const nothingTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const nothingType = new AST.GenericType("Maybe", [nothingTypeVar], 0, 0)
  env.set("Nothing", nothingType)

  // Just : 'a -> Maybe<'a>
  const justTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const justMaybeType = new AST.GenericType("Maybe", [justTypeVar], 0, 0)
  const justType = new AST.FunctionType(justTypeVar, justMaybeType, 0, 0)
  env.set("Just", justType)

  // Signal constructor for reactive programming
  // Signal : 'a -> Signal<'a>
  const signalTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const signalReturnType = new AST.GenericType("Signal", [signalTypeVar], 0, 0)
  const signalConstructorType = new AST.FunctionType(
    signalTypeVar,
    signalReturnType,
    0,
    0
  )
  env.set("Signal", signalConstructorType)

  // Either constructors for pattern matching and expressions
  // Left : 'a -> Either<'a, 'b>
  const leftTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const leftRightTypeVar = new PolymorphicTypeVariable("b", 0, 0)
  const leftEitherType = new AST.GenericType(
    "Either",
    [leftTypeVar, leftRightTypeVar],
    0,
    0
  )
  const leftType = new AST.FunctionType(leftTypeVar, leftEitherType, 0, 0)
  env.set("Left", leftType)

  // Right : 'b -> Either<'a, 'b>
  const rightLeftTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const rightTypeVar = new PolymorphicTypeVariable("b", 0, 0)
  const rightEitherType = new AST.GenericType(
    "Either",
    [rightLeftTypeVar, rightTypeVar],
    0,
    0
  )
  const rightType = new AST.FunctionType(rightTypeVar, rightEitherType, 0, 0)
  env.set("Right", rightType)

  // Task constructor for pattern matching and expressions
  // Task : (() -> Promise<'a>) -> Task<'a>
  const taskTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const promiseType = new AST.GenericType("Promise", [taskTypeVar], 0, 0)
  const computationType = new AST.FunctionType(
    new AST.PrimitiveType("Unit", 0, 0), // () ->
    promiseType, // Promise<'a>
    0,
    0
  )
  const taskResultType = new AST.GenericType("Task", [taskTypeVar], 0, 0)
  const taskType = new AST.FunctionType(computationType, taskResultType, 0, 0)
  env.set("Task", taskType)

  // resolve function: 'a -> (() -> Promise<'a>)
  const resolveTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const resolvePromiseType = new AST.GenericType(
    "Promise",
    [resolveTypeVar],
    0,
    0
  )
  const resolveComputationType = new AST.FunctionType(
    new AST.PrimitiveType("Unit", 0, 0),
    resolvePromiseType,
    0,
    0
  )
  const resolveType = new AST.FunctionType(
    resolveTypeVar,
    resolveComputationType,
    0,
    0
  )
  env.set("resolve", resolveType)

  // run function: Task<'a> -> Promise<'a>
  const runTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const runTaskType = new AST.GenericType("Task", [runTypeVar], 0, 0)
  const runPromiseType = new AST.GenericType("Promise", [runTypeVar], 0, 0)
  const runType = new AST.FunctionType(runTaskType, runPromiseType, 0, 0)
  env.set("run", runType)
  env.set("ssrgRun", runType)

  // ssrgTryRun function: Task<'a> -> Promise<Either<String, 'a>>
  const tryRunTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const tryRunInputTaskType = new AST.GenericType("Task", [tryRunTypeVar], 0, 0)
  const tryRunEitherType = new AST.GenericType(
    "Either",
    [new AST.PrimitiveType("String", 0, 0), tryRunTypeVar],
    0,
    0
  )
  const tryRunOutputPromiseType = new AST.GenericType(
    "Promise",
    [tryRunEitherType],
    0,
    0
  )
  const tryRunType = new AST.FunctionType(
    tryRunInputTaskType,
    tryRunOutputPromiseType,
    0,
    0
  )
  env.set("ssrgTryRun", tryRunType)
  env.set("tryRun", tryRunType)

  // reject function: 'a -> (() -> Promise<'b>)
  const rejectTypeVar = new PolymorphicTypeVariable("a", 0, 0)
  const rejectResultTypeVar = new PolymorphicTypeVariable("b", 0, 0)
  const rejectPromiseType = new AST.GenericType(
    "Promise",
    [rejectResultTypeVar],
    0,
    0
  )
  const rejectComputationType = new AST.FunctionType(
    new AST.PrimitiveType("Unit", 0, 0),
    rejectPromiseType,
    0,
    0
  )
  const rejectType = new AST.FunctionType(
    rejectTypeVar,
    rejectComputationType,
    0,
    0
  )
  env.set("reject", rejectType)

  // Task Functor operation: <$>
  // (<$>) : ('a -> 'b) -> Task<'a> -> Task<'b>
  const fmapA = new PolymorphicTypeVariable("a", 0, 0)
  const fmapB = new PolymorphicTypeVariable("b", 0, 0)
  const fmapFunc = new AST.FunctionType(fmapA, fmapB, 0, 0)
  const fmapTaskA = new AST.GenericType("Task", [fmapA], 0, 0)
  const fmapTaskB = new AST.GenericType("Task", [fmapB], 0, 0)
  const fmapCurried = new AST.FunctionType(fmapTaskA, fmapTaskB, 0, 0)
  const fmapTaskType = new AST.FunctionType(fmapFunc, fmapCurried, 0, 0)
  env.set("mapTask", fmapTaskType)

  // Task Applicative operation: <*>
  // (<*>) : Task<'a -> 'b> -> Task<'a> -> Task<'b>
  const applyA = new PolymorphicTypeVariable("a", 0, 0)
  const applyB = new PolymorphicTypeVariable("b", 0, 0)
  const applyFunc = new AST.FunctionType(applyA, applyB, 0, 0)
  const applyTaskFunc = new AST.GenericType("Task", [applyFunc], 0, 0)
  const applyTaskA = new AST.GenericType("Task", [applyA], 0, 0)
  const applyTaskB = new AST.GenericType("Task", [applyB], 0, 0)
  const applyCurried = new AST.FunctionType(applyTaskA, applyTaskB, 0, 0)
  const applyTaskType = new AST.FunctionType(applyTaskFunc, applyCurried, 0, 0)
  env.set("applyTask", applyTaskType)

  // Task Monad operation: >>=
  // (>>=) : Task<'a> -> ('a -> Task<'b>) -> Task<'b>
  const bindA = new PolymorphicTypeVariable("a", 0, 0)
  const bindB = new PolymorphicTypeVariable("b", 0, 0)
  const bindTaskA = new AST.GenericType("Task", [bindA], 0, 0)
  const bindTaskB = new AST.GenericType("Task", [bindB], 0, 0)
  const bindFunc = new AST.FunctionType(bindA, bindTaskB, 0, 0)
  const bindResult = new AST.FunctionType(bindFunc, bindTaskB, 0, 0)
  const bindTaskType = new AST.FunctionType(bindTaskA, bindResult, 0, 0)
  env.set("bindTask", bindTaskType)

  // Boolean constants
  const boolType = new AST.PrimitiveType("Bool", 0, 0)
  env.set("true", boolType)
  env.set("false", boolType)

  // typeof: 'a -> String (多相関数)
  const typeofType = new AST.FunctionType(
    new PolymorphicTypeVariable("a", 0, 0),
    new AST.PrimitiveType("String", 0, 0),
    0,
    0
  )
  env.set("typeof", typeofType)

  // typeof': 'a -> String (エイリアス情報付き)
  const typeofWithAliasesType = new AST.FunctionType(
    new PolymorphicTypeVariable("a", 0, 0),
    new AST.PrimitiveType("String", 0, 0),
    0,
    0
  )
  env.set("typeof'", typeofWithAliasesType)

  return env
}

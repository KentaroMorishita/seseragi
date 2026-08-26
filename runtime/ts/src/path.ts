import { type Either, Just, Left, type Maybe, Nothing, Right } from "./sum"

const pathBrand: unique symbol = Symbol("seseragi.path")
const pathValues = new WeakMap<object, string>()

export type Path = Readonly<{ readonly [pathBrand]: true }>

export type EmptyPath = Readonly<{ tag: "EmptyPath" }>
export type PathContainsNul = Readonly<{
  tag: "PathContainsNul"
  value: Readonly<{ offset: number }>
}>
export type PathContainsBackslash = Readonly<{
  tag: "PathContainsBackslash"
  value: Readonly<{ offset: number }>
}>
export type InvalidDriveRoot = Readonly<{ tag: "InvalidDriveRoot" }>
export type InvalidUncRoot = Readonly<{ tag: "InvalidUncRoot" }>
export type InvalidPathSegment = Readonly<{
  tag: "InvalidPathSegment"
  value: string
}>
export type AbsoluteChildPath = Readonly<{ tag: "AbsoluteChildPath" }>

export type PathError =
  | EmptyPath
  | PathContainsNul
  | PathContainsBackslash
  | InvalidDriveRoot
  | InvalidUncRoot
  | InvalidPathSegment
  | AbsoluteChildPath

export const EmptyPath: EmptyPath = Object.freeze({ tag: "EmptyPath" })
export const InvalidDriveRoot: InvalidDriveRoot = Object.freeze({
  tag: "InvalidDriveRoot",
})
export const InvalidUncRoot: InvalidUncRoot = Object.freeze({
  tag: "InvalidUncRoot",
})
export const AbsoluteChildPath: AbsoluteChildPath = Object.freeze({
  tag: "AbsoluteChildPath",
})

export function PathContainsNul(value: {
  readonly offset: number
}): PathContainsNul {
  return Object.freeze({ tag: "PathContainsNul", value: Object.freeze(value) })
}

export function PathContainsBackslash(value: {
  readonly offset: number
}): PathContainsBackslash {
  return Object.freeze({
    tag: "PathContainsBackslash",
    value: Object.freeze(value),
  })
}

export function InvalidPathSegment(value: string): InvalidPathSegment {
  return Object.freeze({ tag: "InvalidPathSegment", value })
}

type PathParts = Readonly<{
  root: "relative" | "posix" | "drive" | "unc"
  prefix: string
  segments: ReadonlyArray<string>
}>

export function parse(text: string): Either<PathError, Path> {
  const error = validate(text)
  return error === undefined ? Right(pathValue(text)) : Left(error)
}

/** Internal trusted constructor used only after a Provider path codec validates. */
export function pathFromProvider(text: string): Path {
  const parsed = parse(text)
  if (parsed.tag === "Left") {
    throw new TypeError(
      `provider returned an invalid portable path: ${parsed.value.tag}`
    )
  }
  return parsed.value
}

export function render(value: Path): string {
  const text = pathValues.get(value)
  if (text === undefined) throw new TypeError("Path value is invalid")
  return text
}

export function current(): Path {
  return pathValue(".")
}

export function isAbsolute(value: Path): boolean {
  return parts(render(value)).root !== "relative"
}

export function normalize(value: Path): Path {
  const parsed = parts(render(value))
  const normalized: string[] = []
  for (const segment of parsed.segments) {
    if (segment.length === 0 || segment === ".") continue
    if (segment === "..") {
      const previous = normalized.at(-1)
      if (previous !== undefined && previous !== "..") {
        normalized.pop()
      } else if (parsed.root === "relative") {
        normalized.push(segment)
      }
      continue
    }
    normalized.push(segment)
  }
  if (normalized.length === 0) {
    return parsed.root === "relative" ? current() : pathValue(parsed.prefix)
  }
  return pathValue(`${parsed.prefix}${normalized.join("/")}`)
}

export function join(childPath: Path, base: Path): Either<PathError, Path> {
  if (isAbsolute(childPath)) return Left(AbsoluteChildPath)
  const childText = render(childPath)
  if (childText === ".") return Right(base)
  const baseText = render(base)
  if (baseText === ".") return Right(childPath)
  return Right(pathValue(appendPath(baseText, childText)))
}

export function child(name: string, base: Path): Either<PathError, Path> {
  if (
    name.length === 0 ||
    name === "." ||
    name === ".." ||
    name.includes("/") ||
    name.includes("\\") ||
    name.includes("\0")
  ) {
    return Left(InvalidPathSegment(name))
  }
  return Right(pathValue(appendPath(render(base), name)))
}

export function parent(value: Path): Maybe<Path> {
  const parsed = parts(render(value))
  if (
    (parsed.root === "relative" && render(value) === ".") ||
    parsed.segments.length === 0
  ) {
    return Nothing
  }
  const segments = [...parsed.segments]
  segments.pop()
  if (segments.length === 0) {
    return parsed.root === "relative"
      ? Just(current())
      : Just(pathValue(parsed.prefix))
  }
  return Just(pathValue(`${parsed.prefix}${segments.join("/")}`))
}

export function fileName(value: Path): Maybe<string> {
  const parsed = parts(render(value))
  if (parsed.segments.length === 0) return Nothing
  const name = parsed.segments.at(-1)
  return name === undefined || name.length === 0 ? Nothing : Just(name)
}

export function extension(value: Path): Maybe<string> {
  const name = fileName(value)
  if (name.tag === "Nothing") return Nothing
  const index = name.value.lastIndexOf(".")
  return index <= 0 || index === name.value.length - 1
    ? Nothing
    : Just(name.value.slice(index + 1))
}

function validate(text: string): PathError | undefined {
  if (text.length === 0) return EmptyPath
  const nul = text.indexOf("\0")
  if (nul >= 0) return PathContainsNul({ offset: nul })
  const backslash = text.indexOf("\\")
  if (backslash >= 0) return PathContainsBackslash({ offset: backslash })
  if (/^[A-Za-z]:/.test(text) && !/^[A-Za-z]:\//.test(text)) {
    return InvalidDriveRoot
  }
  if (text.startsWith("//")) {
    const segments = text.slice(2).split("/")
    if (
      segments.length < 2 ||
      segments[0]?.length === 0 ||
      segments[1]?.length === 0 ||
      segments[0] === "." ||
      segments[0] === ".." ||
      segments[1] === "." ||
      segments[1] === ".."
    ) {
      return InvalidUncRoot
    }
  }
  return undefined
}

function parts(text: string): PathParts {
  if (text.startsWith("//")) {
    const values = text.slice(2).split("/")
    const server = values.shift() as string
    const share = values.shift() as string
    return {
      root: "unc",
      prefix: `//${server}/${share}/`,
      segments: trimTrailingEmpty(values),
    }
  }
  if (/^[A-Za-z]:\//.test(text)) {
    return {
      root: "drive",
      prefix: text.slice(0, 3),
      segments: trimTrailingEmpty(text.slice(3).split("/")),
    }
  }
  if (text.startsWith("/")) {
    return {
      root: "posix",
      prefix: "/",
      segments: trimTrailingEmpty(text.slice(1).split("/")),
    }
  }
  return {
    root: "relative",
    prefix: "",
    segments: trimTrailingEmpty(text.split("/")),
  }
}

function trimTrailingEmpty(values: string[]): ReadonlyArray<string> {
  while (values.at(-1) === "") values.pop()
  return values
}

function appendPath(base: string, suffix: string): string {
  return `${base.replace(/\/+$/, "")}/${suffix}`
}

function pathValue(text: string): Path {
  const value = Object.create(null) as Path
  Object.defineProperty(value, pathBrand, { enumerable: false, value: true })
  pathValues.set(value, text)
  return Object.freeze(value)
}

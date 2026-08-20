import type { Effect, EffectContext, Unit } from "./effect"
import type { WebUrl } from "./html"
import { parseWebUrl } from "./html"
import type { ServiceResult } from "./service"
import { serviceEffect, serviceFailure, serviceSuccess } from "./service"
import { type MutableSignal, make, type Signal, set } from "./signal"
import { type Either, Just, Left, type Maybe, Nothing, Right } from "./sum"

const urlBrand = Symbol("seseragi.navigation.url")
const queryBrand = Symbol("seseragi.navigation.query")
const locationBrand = Symbol("seseragi.navigation.location")

export type Url = Readonly<{
  readonly [urlBrand]: true
  readonly href: string
}>

export type Query = Readonly<{
  readonly [queryBrand]: true
  readonly entries: ReadonlyArray<Readonly<{ name: string; value: string }>>
}>

export type Location = Readonly<{
  readonly [locationBrand]: true
  readonly url: Url
}>

export type UrlBuildError =
  | Readonly<{ readonly tag: "InvalidUrl"; readonly value: { offset: number } }>
  | Readonly<{ readonly tag: "UnsupportedUrlScheme"; readonly value: string }>
  | Readonly<{ readonly tag: "UrlContainsUserInfo" }>
  | Readonly<{
      readonly tag: "InvalidPercentEncoding"
      readonly value: { offset: number }
    }>

export type NavigationError =
  | Readonly<{
      readonly tag: "CrossOriginNavigation"
      readonly value: { expected: string; actual: string }
    }>
  | Readonly<{ readonly tag: "NavigationUnavailable"; readonly value: string }>
  | Readonly<{
      readonly tag: "NavigationSecurityFailure"
      readonly value: string
    }>

export const InvalidUrl = (value: { offset: number }): UrlBuildError =>
  Object.freeze({ tag: "InvalidUrl", value: Object.freeze({ ...value }) })

export const UnsupportedUrlScheme = (value: string): UrlBuildError =>
  Object.freeze({ tag: "UnsupportedUrlScheme", value })

export const UrlContainsUserInfo: UrlBuildError = Object.freeze({
  tag: "UrlContainsUserInfo",
})

export const InvalidPercentEncoding = (value: {
  offset: number
}): UrlBuildError =>
  Object.freeze({
    tag: "InvalidPercentEncoding",
    value: Object.freeze({ ...value }),
  })

export const CrossOriginNavigation = (value: {
  expected: string
  actual: string
}): NavigationError =>
  Object.freeze({
    tag: "CrossOriginNavigation",
    value: Object.freeze({ ...value }),
  })

export const NavigationUnavailable = (value: string): NavigationError =>
  Object.freeze({ tag: "NavigationUnavailable", value })

export const NavigationSecurityFailure = (value: string): NavigationError =>
  Object.freeze({ tag: "NavigationSecurityFailure", value })

export type Navigation = Readonly<{
  current: (
    context: EffectContext
  ) => Promise<ServiceResult<NavigationError, Location>>
  push: (
    url: Url,
    context: EffectContext
  ) => Promise<ServiceResult<NavigationError, Location>>
  replace: (
    url: Url,
    context: EffectContext
  ) => Promise<ServiceResult<NavigationError, Location>>
  back: (
    context: EffectContext
  ) => Promise<ServiceResult<NavigationError, Unit>>
  forward: (
    context: EffectContext
  ) => Promise<ServiceResult<NavigationError, Unit>>
  nextChange: (context: EffectContext) => Promise<Location>
}>

export type NavigationEnvironment = Readonly<{ navigation: Navigation }>

export const emptyQuery: Query = freezeQuery([])

export function parseUrl(value: string): Either<UrlBuildError, Url> {
  return parseAbsoluteOrResolved(value)
}

export function resolveUrl(
  reference: string,
  base: Url
): Either<UrlBuildError, Url> {
  return parseAbsoluteOrResolved(reference, base.href)
}

export function renderUrl(value: Url): string {
  return value.href
}

export function urlOrigin(value: Url): string {
  return new URL(value.href).origin
}

export function pathSegments(value: Url): ReadonlyArray<string> {
  const pathname = new URL(value.href).pathname
  if (pathname === "/") return Object.freeze([])
  return Object.freeze(
    pathname
      .slice(1)
      .split("/")
      .map((segment) => decodeURIComponent(segment))
  )
}

export function withPathSegments(
  segments: ReadonlyArray<string>,
  value: Url
): Url {
  const parsed = new URL(value.href)
  parsed.pathname =
    segments.length === 0
      ? "/"
      : `/${segments.map((segment) => encodeURIComponent(segment)).join("/")}`
  return freezeUrl(parsed.href)
}

export function urlQuery(value: Url): Query {
  return queryFromSearchParams(new URL(value.href).searchParams)
}

export function withQuery(query: Query, value: Url): Url {
  const parsed = new URL(value.href)
  const rendered = renderQuery(query)
  parsed.search = rendered.length === 0 ? "" : `?${rendered}`
  return freezeUrl(parsed.href)
}

export function urlFragment(value: Url): Maybe<string> {
  const fragment = new URL(value.href).hash
  return fragment.length === 0
    ? Nothing
    : Just(decodeURIComponent(fragment.slice(1)))
}

export function withFragment(fragment: string, value: Url): Url {
  const parsed = new URL(value.href)
  parsed.hash = encodeURIComponent(fragment)
  return freezeUrl(parsed.href)
}

export function withoutFragment(value: Url): Url {
  const parsed = new URL(value.href)
  parsed.hash = ""
  return freezeUrl(parsed.href)
}

export function parseQuery(value: string): Either<UrlBuildError, Query> {
  const text = value.startsWith("?") ? value.slice(1) : value
  const invalid = invalidPercentOffset(text)
  if (invalid !== undefined)
    return Left(InvalidPercentEncoding({ offset: invalid }))
  return Right(queryFromSearchParams(new URLSearchParams(text)))
}

export function appendQuery(name: string, value: string, query: Query): Query {
  return freezeQuery([...query.entries, Object.freeze({ name, value })])
}

export function setQuery(name: string, value: string, query: Query): Query {
  const first = query.entries.findIndex((entry) => entry.name === name)
  const filtered = query.entries.filter((entry) => entry.name !== name)
  const index = first < 0 ? filtered.length : first
  return freezeQuery([
    ...filtered.slice(0, index),
    Object.freeze({ name, value }),
    ...filtered.slice(index),
  ])
}

export function removeQuery(name: string, query: Query): Query {
  return freezeQuery(query.entries.filter((entry) => entry.name !== name))
}

export function queryValues(name: string, query: Query): ReadonlyArray<string> {
  return Object.freeze(
    query.entries
      .filter((entry) => entry.name === name)
      .map((entry) => entry.value)
  )
}

export function queryEntries(
  query: Query
): ReadonlyArray<readonly [string, string]> {
  return Object.freeze(
    query.entries.map(({ name, value }) =>
      Object.freeze([name, value] as const)
    )
  )
}

export function renderQuery(query: Query): string {
  const params = new URLSearchParams()
  for (const { name, value } of query.entries) params.append(name, value)
  return params.toString()
}

export function toWebUrl(value: Url): WebUrl {
  const parsed = parseWebUrl(value.href)
  if (parsed.tag === "Left") {
    throw new TypeError("normalized navigation URL must be a valid WebUrl")
  }
  return parsed.value
}

export function locationUrl(value: Location): Url {
  return value.url
}

export function current(
  _unit?: Unit
): Effect<NavigationEnvironment, NavigationError, Location> {
  return serviceEffect((environment, context) =>
    environment.navigation.current(context)
  )
}

export function push(
  value: Url
): Effect<NavigationEnvironment, NavigationError, Location> {
  return serviceEffect((environment, context) =>
    environment.navigation.push(value, context)
  )
}

export function replace(
  value: Url
): Effect<NavigationEnvironment, NavigationError, Location> {
  return serviceEffect((environment, context) =>
    environment.navigation.replace(value, context)
  )
}

export function back(
  _unit?: Unit
): Effect<NavigationEnvironment, NavigationError, Unit> {
  return serviceEffect((environment, context) =>
    environment.navigation.back(context)
  )
}

export function forward(
  _unit?: Unit
): Effect<NavigationEnvironment, NavigationError, Unit> {
  return serviceEffect((environment, context) =>
    environment.navigation.forward(context)
  )
}

export function locationSignal(
  _unit?: Unit
): Effect<NavigationEnvironment, NavigationError, Signal<Location>> {
  return serviceEffect(async (environment, context) => {
    const initial = await environment.navigation.current(context)
    if (initial.kind === "failure") return initial
    const source = await make(initial.value)(environment, context)
    void pumpLocationChanges(environment, context, source)
    return serviceSuccess(source)
  })
}

export function errorMessage(error: NavigationError): string {
  if (error.tag === "CrossOriginNavigation") {
    return `${error.tag}: expected ${error.value.expected}, actual ${error.value.actual}`
  }
  return `${error.tag}: ${error.value}`
}

export function navigationSuccess<Success>(
  value: Success
): ServiceResult<never, Success> {
  return serviceSuccess(value)
}

export function navigationFailure(
  error: NavigationError
): ServiceResult<NavigationError, never> {
  return serviceFailure(error)
}

export function locationFromHref(href: string): Location {
  const parsed = parseUrl(href)
  if (parsed.tag === "Left") {
    throw new TypeError("navigation provider returned an invalid location URL")
  }
  return Object.freeze({
    [locationBrand]: true as const,
    url: parsed.value,
  })
}

async function pumpLocationChanges(
  environment: NavigationEnvironment,
  context: EffectContext,
  source: MutableSignal<Location>
): Promise<void> {
  try {
    while (!context.cancelled) {
      const next = await environment.navigation.nextChange(context)
      await set(next, source)(environment, context)
    }
  } catch {
    // A Signal cannot acquire a new typed failure after construction. Provider
    // shutdown and caller cancellation therefore terminate this detached pump;
    // bridge defects remain observable at the operation boundary that created
    // the Signal.
  }
}

function parseAbsoluteOrResolved(
  value: string,
  base?: string
): Either<UrlBuildError, Url> {
  const invalid = invalidPercentOffset(value)
  if (invalid !== undefined)
    return Left(InvalidPercentEncoding({ offset: invalid }))
  let parsed: URL
  try {
    parsed = base === undefined ? new URL(value) : new URL(value, base)
  } catch {
    return Left(InvalidUrl({ offset: 0 }))
  }
  const scheme = parsed.protocol.slice(0, -1).toLowerCase()
  if (scheme !== "http" && scheme !== "https") {
    return Left(UnsupportedUrlScheme(scheme))
  }
  if (parsed.username.length > 0 || parsed.password.length > 0) {
    return Left(UrlContainsUserInfo)
  }
  return Right(freezeUrl(parsed.href))
}

function invalidPercentOffset(value: string): number | undefined {
  for (let index = 0; index < value.length; index += 1) {
    if (value[index] !== "%") continue
    if (!/^[0-9a-fA-F]{2}$/.test(value.slice(index + 1, index + 3))) {
      return index
    }
    index += 2
  }
  return undefined
}

function freezeUrl(href: string): Url {
  return Object.freeze({ [urlBrand]: true as const, href })
}

function freezeQuery(
  entries: ReadonlyArray<Readonly<{ name: string; value: string }>>
): Query {
  return Object.freeze({
    [queryBrand]: true as const,
    entries: Object.freeze(
      entries.map(({ name, value }) => Object.freeze({ name, value }))
    ),
  })
}

function queryFromSearchParams(params: URLSearchParams): Query {
  return freezeQuery([...params].map(([name, value]) => ({ name, value })))
}

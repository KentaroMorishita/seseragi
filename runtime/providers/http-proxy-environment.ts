import { BlockList, isIP } from "node:net"

type Environment = Readonly<Record<string, string | undefined>>

export function selectHttpProxy(
  destination: URL,
  environment: Environment = process.env
): URL | undefined {
  const selected =
    (destination.protocol === "https:"
      ? environment.https_proxy || environment.HTTPS_PROXY
      : environment.http_proxy || environment.HTTP_PROXY) ||
    environment.all_proxy ||
    environment.ALL_PROXY
  if (!selected) return undefined
  const bypass = environment.no_proxy || environment.NO_PROXY
  if (bypass && bypassesProxy(destination, bypass)) return undefined
  let proxy: URL
  try {
    proxy = new URL(selected)
  } catch {
    throw new TypeError("HTTP proxy URL is invalid")
  }
  if (proxy.protocol !== "http:" && proxy.protocol !== "https:") {
    throw new TypeError("HTTP proxy protocol is unsupported")
  }
  return proxy
}

function bypassesProxy(destination: URL, bypass: string): boolean {
  const hostname = trimRootDot(unbracket(destination.hostname).toLowerCase())
  const port =
    destination.port || (destination.protocol === "https:" ? "443" : "80")
  for (const token of bypass.split(/[,\s]+/)) {
    if (token === "*") return true
    if (token === "") continue
    let pattern = token.toLowerCase()
    if (pattern.includes("/")) {
      if (matchesSubnet(hostname, pattern)) return true
      continue
    }
    let patternPort: string | undefined
    if (pattern.startsWith("[")) {
      const end = pattern.indexOf("]")
      if (end < 0) continue
      const suffix = pattern.slice(end + 1)
      if (suffix !== "" && !/^:[0-9]+$/.test(suffix)) continue
      patternPort = suffix === "" ? undefined : suffix.slice(1)
      pattern = pattern.slice(1, end)
    } else if (isIP(pattern) === 0) {
      const split = pattern.lastIndexOf(":")
      if (split >= 0) {
        // IPv6 ports require brackets; an invalid bare IPv6 token is not a
        // host:port rule for the shorter IPv6 address before its last colon.
        if (split !== pattern.indexOf(":")) continue
        patternPort = pattern.slice(split + 1)
        pattern = pattern.slice(0, split)
        if (!/^[0-9]+$/.test(patternPort)) continue
      }
    }
    if (patternPort !== undefined && Number(patternPort) !== Number(port))
      continue
    pattern = trimRootDot(pattern)
    if (isIP(hostname) !== 0 || isIP(pattern) !== 0) {
      if (matchesAddress(hostname, pattern)) return true
      continue
    }
    if (pattern.startsWith("*.")) pattern = pattern.slice(2)
    else if (pattern.startsWith(".")) pattern = pattern.slice(1)
    if (
      pattern !== "" &&
      (hostname === pattern || hostname.endsWith(`.${pattern}`))
    ) {
      return true
    }
  }
  return false
}

function matchesAddress(hostname: string, address: string): boolean {
  const family = isIP(address)
  if (family === 0 || family !== isIP(hostname)) return false
  const blockList = new BlockList()
  const type = family === 4 ? "ipv4" : "ipv6"
  blockList.addAddress(address, type)
  return blockList.check(hostname, type)
}

function matchesSubnet(hostname: string, pattern: string): boolean {
  const [subnet, prefix, ...extra] = pattern.split("/")
  if (extra.length !== 0 || subnet === undefined || prefix === undefined)
    return false
  if (!/^[0-9]+$/.test(prefix)) return false
  const address = unbracket(subnet)
  const family = isIP(address)
  if (family === 0 || family !== isIP(hostname)) return false
  const length = Number(prefix)
  if (length > (family === 4 ? 32 : 128)) return false
  try {
    const blockList = new BlockList()
    const type = family === 4 ? "ipv4" : "ipv6"
    blockList.addSubnet(address, length, type)
    return blockList.check(hostname, type)
  } catch {
    return false
  }
}

function unbracket(value: string): string {
  return value.startsWith("[") && value.endsWith("]")
    ? value.slice(1, -1)
    : value
}

function trimRootDot(value: string): string {
  return value.endsWith(".") ? value.slice(0, -1) : value
}

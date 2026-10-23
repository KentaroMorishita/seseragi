import { expect, test } from "bun:test"
import { selectHttpProxy } from "../../providers/http-proxy-environment"

const primary = "http://proxy.invalid:3128/"
const secondary = "https://other.invalid:8443/"

test("Bun helper proxy selection preserves scheme and environment precedence", () => {
  const environment: Record<string, string> = {
    HTTP_PROXY: primary,
    HTTPS_PROXY: secondary,
    ALL_PROXY: "http://fallback.invalid:8080/",
  }
  const http = new URL("http://destination.invalid/")
  const https = new URL("https://destination.invalid/")
  expect(selectHttpProxy(http, environment)?.href).toBe(primary)
  expect(selectHttpProxy(https, environment)?.href).toBe(secondary)
  environment.http_proxy = secondary
  expect(selectHttpProxy(http, environment)?.href).toBe(secondary)
  environment.http_proxy = ""
  expect(selectHttpProxy(http, environment)?.href).toBe(primary)
  delete environment.HTTP_PROXY
  expect(selectHttpProxy(http, environment)?.href).toBe(environment.ALL_PROXY)
  environment.all_proxy = primary
  expect(selectHttpProxy(http, environment)?.href).toBe(primary)
  delete environment.HTTPS_PROXY
  expect(selectHttpProxy(https, environment)?.href).toBe(primary)
  expect(selectHttpProxy(https, { HTTP_PROXY: primary })).toBeUndefined()
  expect(selectHttpProxy(http, { HTTPS_PROXY: secondary })).toBeUndefined()
})

test.each([
  ["http://example.com/", "*", true],
  ["http://example.com/", "example.com", true],
  ["http://sub.example.com/", "example.com", true],
  ["http://example.com/", ".example.com", true],
  ["http://example.com/", "*.example.com", true],
  ["http://sub.example.com/", "*.example.com", true],
  ["http://EXAMPLE.COM/", "Example.Com", true],
  ["http://example.com/", "example.com.", true],
  ["http://example.com/", ".example.com.", true],
  ["http://example.com./", "example.com", true],
  ["http://notexample.com/", "example.com", false],
  ["http://example.com.attacker.invalid/", "example.com", false],
  ["http://example.com/", "other.invalid,example.com", true],
  ["http://example.com/", "other.invalid\t example.com\n", true],
  ["http://example.com/", "example.com:80", true],
  ["http://example.com/", "example.com:080", true],
  ["https://example.com/", "example.com:443", true],
  ["http://example.com:8080/", "example.com:8080", true],
  ["http://example.com:8080/", "example.com:80", false],
  ["http://example.com/", "example.com:", false],
  ["http://127.0.0.1/", "127.0.0.1", true],
  ["http://127.0.0.1/", "127.0.0.0/8", true],
  ["http://128.0.0.1/", "127.0.0.0/8", false],
  ["http://127.0.0.1:8080/", "127.0.0.1:8080", true],
  ["http://127.0.0.1:8080/", "127.0.0.1:80", false],
  ["http://[::1]/", "::1", true],
  ["http://[::1]/", "[::1]", true],
  ["http://[::1]/", "0:0:0:0:0:0:0:1", true],
  ["http://[::1]:8080/", "[::1]:8080", true],
  ["http://[::1]:8080/", "[::1]:80", false],
  ["http://[::1]:52129/", "::1:52129", false],
  ["http://[fd00::1]/", "fd00::/8", true],
  ["http://[fe00::1]/", "fd00::/8", false],
  ["http://127.0.0.1/", "127.0.0.0/99", false],
  ["http://127.0.0.1/", "127.0.0.0/8/1", false],
  ["http://[::1]/", "[::1", false],
  ["http://example.com/", ".", false],
])("proxy bypass %s / %s = %s", (destination, rule, bypasses) => {
  const proxy = selectHttpProxy(new URL(destination as string), {
    HTTP_PROXY: primary,
    HTTPS_PROXY: primary,
    NO_PROXY: rule as string,
  })
  expect(proxy?.href).toBe(bypasses ? undefined : primary)
})

test("lowercase no_proxy wins and empty falls back without caching", () => {
  const destination = new URL("http://example.com/")
  const environment = {
    HTTP_PROXY: primary,
    NO_PROXY: "example.com",
    no_proxy: "other.invalid",
  }
  expect(selectHttpProxy(destination, environment)?.href).toBe(primary)
  environment.no_proxy = ""
  expect(selectHttpProxy(destination, environment)).toBeUndefined()
  environment.NO_PROXY = ""
  expect(selectHttpProxy(destination, environment)?.href).toBe(primary)
})

test("invalid proxy configuration fails without exposing credentials", () => {
  const destination = new URL("http://example.com/")
  for (const proxy of [
    "invalid-secret",
    "socks5://user:secret@proxy.invalid",
  ]) {
    expect(() => selectHttpProxy(destination, { HTTP_PROXY: proxy })).toThrow(
      /^HTTP proxy (URL is invalid|protocol is unsupported)$/
    )
  }
})

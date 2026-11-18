import { describe, expect, test } from "bun:test"
import { createPreviewDocument } from "../src/preview-document"
import {
  createPreviewLinkOpener,
  externalPreviewLink,
  installPreviewLinkBridge,
  previewLinkAction,
  staticPreviewLinkScript,
} from "../src/runtime/preview-links"

describe("external preview links", () => {
  test("accept only absolute http and https URLs", () => {
    expect(externalPreviewLink("https://github.com/KentaroMorishita")).toBe(
      "https://github.com/KentaroMorishita"
    )
    expect(externalPreviewLink(" http://example.com/a?b=1#c ")).toBe(
      "http://example.com/a?b=1#c"
    )
    for (const href of [
      null,
      "",
      "/notes",
      "#top",
      "notes/1",
      "javascript:alert(1)",
      "data:text/html,hi",
      "blob:https://example.com/x",
      "mailto:a@example.com",
      "https://user:pass@example.com/",
      `https://example.com/${"a".repeat(2100)}`,
    ])
      expect(externalPreviewLink(href)).toBeUndefined()
  })

  test("open the Playground itself in place and everything else in a new tab", () => {
    const origin = "https://seseragi.vercel.app"
    expect(previewLinkAction("https://seseragi.vercel.app/", origin)).toEqual({
      kind: "navigate",
      url: "https://seseragi.vercel.app/",
    })
    expect(
      previewLinkAction("https://seseragi.vercel.app/tour/", origin).kind
    ).toBe("navigate")
    expect(
      previewLinkAction("https://seseragi.vercel.app/deep-dive/", origin).kind
    ).toBe("new-tab")
    expect(previewLinkAction("https://github.com/", origin).kind).toBe(
      "new-tab"
    )
    expect(previewLinkAction("javascript:alert(1)", origin).kind).toBe("ignore")
    expect(previewLinkAction(42, origin).kind).toBe("ignore")
  })
})

type Listener = (event: Event) => void

function fakeWindow() {
  const listeners = new Map<string, Listener>()
  return {
    listeners,
    addEventListener: (type: string, listener: Listener) =>
      void listeners.set(type, listener),
    removeEventListener: (type: string) => void listeners.delete(type),
  } as unknown as Window & { listeners: Map<string, Listener> }
}

function click(
  href: string | null,
  options: Partial<{
    isTrusted: boolean
    defaultPrevented: boolean
    type: string
    button: number
  }> = {}
) {
  const anchor =
    href === null
      ? null
      : { getAttribute: (name: string) => (name === "href" ? href : null) }
  let prevented = false
  return {
    event: {
      type: options.type ?? "click",
      button: options.button ?? 0,
      isTrusted: options.isTrusted ?? true,
      defaultPrevented: options.defaultPrevented ?? false,
      target: { closest: () => anchor },
      preventDefault: () => {
        prevented = true
      },
    } as unknown as Event,
    prevented: () => prevented,
  }
}

describe("preview link bridge", () => {
  test("forwards a link the user clicked and stops the blocked navigation", () => {
    const target = fakeWindow()
    const sent: string[] = []
    installPreviewLinkBridge(target, (url) => sent.push(url))
    const external = click("https://github.com/")
    target.listeners.get("click")!(external.event)
    expect(sent).toEqual(["https://github.com/"])
    expect(external.prevented()).toBe(true)
  })

  test("leaves program navigation and synthetic clicks alone", () => {
    const target = fakeWindow()
    const sent: string[] = []
    installPreviewLinkBridge(target, (url) => sent.push(url))
    for (const { event } of [
      click("/notes"),
      click("#section"),
      click("https://github.com/", { isTrusted: false }),
      click("https://github.com/", { defaultPrevented: true }),
      click(null),
      click("https://github.com/", { type: "auxclick", button: 2 }),
    ])
      target.listeners.get(event.type)!(event)
    expect(sent).toEqual([])
  })

  test("also opens a middle-clicked link", () => {
    const target = fakeWindow()
    const sent: string[] = []
    installPreviewLinkBridge(target, (url) => sent.push(url))
    const middle = click("https://example.com/", {
      type: "auxclick",
      button: 1,
    })
    target.listeners.get("auxclick")!(middle.event)
    expect(sent).toEqual(["https://example.com/"])
  })
})

describe("preview link opener", () => {
  function shell(origin = "https://seseragi.vercel.app") {
    const opened: unknown[][] = []
    const assigned: string[] = []
    const notices: string[] = []
    const opener = createPreviewLinkOpener({
      window: {
        open: (...args: unknown[]) => {
          opened.push(args)
          return null
        },
        location: {
          origin,
          assign: (url: string) => assigned.push(url),
        },
      } as unknown as Window,
      onOpened: (url) => notices.push(url),
    })
    return { opener, opened, assigned, notices }
  }

  test("opens external links in a new tab without opener or referrer", () => {
    const { opener, opened, notices } = shell()
    opener.open("https://github.com/KentaroMorishita/seseragi")
    expect(opened).toEqual([
      [
        "https://github.com/KentaroMorishita/seseragi",
        "_blank",
        "noopener,noreferrer",
      ],
    ])
    // A blocked popup cannot be detected, so a clickable notice is always shown.
    expect(notices).toEqual(["https://github.com/KentaroMorishita/seseragi"])
  })

  test("moves to the Playground itself in the same tab", () => {
    const { opener, opened, assigned } = shell()
    opener.open("https://seseragi.vercel.app/tour/")
    expect(assigned).toEqual(["https://seseragi.vercel.app/tour/"])
    expect(opened).toEqual([])
  })

  test("ignores anything that is not an http or https URL", () => {
    const { opener, opened, assigned } = shell()
    opener.open("javascript:alert(1)")
    opener.open({ href: "https://github.com/" })
    expect(opened).toEqual([])
    expect(assigned).toEqual([])
  })
})

describe("static preview document", () => {
  test("keeps scripts off unless a link token is given", () => {
    const plain = createPreviewDocument("<p>hi</p>")
    expect(plain).toContain("script-src 'none'")
    expect(plain).not.toContain("<script")
  })

  test("runs only the link script under a per-render nonce", () => {
    const token = "8b0f1c2e-1111-4222-8333-944455556666"
    const html = createPreviewDocument(
      '<a href="https://github.com/">GitHub</a><script>alert(1)</script>',
      false,
      token
    )
    expect(html).toContain(`script-src 'nonce-${token}'`)
    expect(html).not.toContain("'unsafe-inline'; style")
    expect(html).toContain(`<script nonce="${token}">`)
    // The program's own script has no nonce, so the policy blocks it.
    expect(html).toContain("<script>alert(1)</script>")
    expect(html.match(/<script nonce=/gu)).toHaveLength(1)
  })

  test("the link script posts only http and https links with its token", () => {
    const token = "t-1"
    const posted: unknown[] = []
    const listeners = new Map<string, (event: unknown) => void>()
    const run = new Function(
      "addEventListener",
      "parent",
      "URL",
      staticPreviewLinkScript(token)
    )
    run(
      (type: string, listener: (event: unknown) => void) =>
        listeners.set(type, listener),
      { postMessage: (message: unknown) => posted.push(message) },
      URL
    )
    const fire = (href: string, extra: Record<string, unknown> = {}) => {
      let prevented = false
      listeners.get("click")!({
        type: "click",
        button: 0,
        isTrusted: true,
        defaultPrevented: false,
        target: {
          closest: () => ({ getAttribute: () => href }),
        },
        preventDefault: () => {
          prevented = true
        },
        ...extra,
      })
      return prevented
    }
    expect(fire("https://github.com/")).toBe(true)
    expect(fire("javascript:alert(1)")).toBe(false)
    expect(fire("/local")).toBe(false)
    expect(fire("https://example.com/", { isTrusted: false })).toBe(false)
    expect(posted).toEqual([
      {
        type: "seseragi-preview-link",
        token: "t-1",
        href: "https://github.com/",
      },
    ])
  })
})

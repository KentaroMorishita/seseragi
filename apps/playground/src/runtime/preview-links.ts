// Preview の中の外部リンクを、シェル（Playground / Tour）で開く（#1375）。
//
// Preview の枠は sandbox="allow-scripts allow-forms" のまま、外へ移動したり
// 新しいタブを開いたりできない。そこで、枠の中で利用者が押した外部リンクを受け取り、
// シェルに知らせる。シェルは URL を確かめてから新しいタブで開く。
//
// - 受け取るのは、絶対の http: / https: の URL だけ。相対パスや # は、プログラムの中の
//   移動として今までどおり扱う。
// - プログラム自身が処理したクリック（preventDefault 済み）と、プログラムが起こした
//   クリック（isTrusted が偽）は受け取らない。

const maxLinkLength = 2048

/** 枠の中のリンクの href から、シェルで開いてよい絶対 URL を取り出す。開かないものは undefined。 */
export function externalPreviewLink(href: string | null): string | undefined {
  if (href === null) return undefined
  const trimmed = href.trim()
  if (trimmed === "" || trimmed.length > maxLinkLength) return undefined
  let url: URL
  try {
    // 基準の URL を渡さないので、相対パスは解けずに例外になる。
    url = new URL(trimmed)
  } catch {
    return undefined
  }
  if (url.protocol !== "http:" && url.protocol !== "https:") return undefined
  if (url.username !== "" || url.password !== "") return undefined
  return url.href
}

export type PreviewLinkAction =
  | Readonly<{ kind: "navigate"; url: string }>
  | Readonly<{ kind: "new-tab"; url: string }>
  | Readonly<{ kind: "ignore" }>

/**
 * シェルが受け取った URL をどう開くかを決める。
 * Playground 自身（同じ origin の / と /tour/ 以下）は今のタブで移動し、
 * それ以外は新しいタブで開く。
 */
export function previewLinkAction(
  href: unknown,
  shellOrigin: string
): PreviewLinkAction {
  if (typeof href !== "string") return { kind: "ignore" }
  const url = externalPreviewLink(href)
  if (url === undefined) return { kind: "ignore" }
  const parsed = new URL(url)
  if (
    parsed.origin === shellOrigin &&
    (parsed.pathname === "/" ||
      parsed.pathname === "/tour" ||
      parsed.pathname.startsWith("/tour/"))
  )
    return { kind: "navigate", url }
  return { kind: "new-tab", url }
}

type LinkEventTarget = Pick<Window, "addEventListener" | "removeEventListener">

/**
 * 枠の中で、利用者が押した外部リンクを受け取る。window の泡立ちの段階で聞くので、
 * プログラム自身がリンクを処理した（preventDefault した）クリックは任せたままにする。
 */
export function installPreviewLinkBridge(
  target: LinkEventTarget,
  send: (url: string) => void
): () => void {
  const onClick = (event: Event): void => {
    const mouse = event as MouseEvent
    if (!mouse.isTrusted || mouse.defaultPrevented) return
    if (mouse.type === "auxclick" && mouse.button !== 1) return
    const origin = mouse.target as Element | null
    const anchor = origin?.closest?.("a[href]") ?? null
    if (anchor === null) return
    const url = externalPreviewLink(anchor.getAttribute("href"))
    if (url === undefined) return
    mouse.preventDefault()
    send(url)
  }
  target.addEventListener("click", onClick)
  target.addEventListener("auxclick", onClick)
  return () => {
    target.removeEventListener("click", onClick)
    target.removeEventListener("auxclick", onClick)
  }
}

/**
 * 静的な HTML の Preview に入れる小さなスクリプト。静的な Preview は script-src を
 * nonce で絞り、この一つだけを動かす。受け取ったリンクは親（シェル）へ token 付きで送る。
 */
export function staticPreviewLinkScript(token: string): string {
  const quoted = JSON.stringify(token).replaceAll("<", "\\u003c")
  return `(function(){var t=${quoted};function h(e){if(!e.isTrusted||e.defaultPrevented)return;if(e.type==="auxclick"&&e.button!==1)return;var a=e.target&&e.target.closest?e.target.closest("a[href]"):null;if(!a)return;var r=(a.getAttribute("href")||"").trim();if(!r||r.length>${maxLinkLength})return;var u;try{u=new URL(r)}catch(x){return}if(u.protocol!=="http:"&&u.protocol!=="https:")return;if(u.username||u.password)return;e.preventDefault();parent.postMessage({type:"seseragi-preview-link",token:t,href:u.href},"*")}addEventListener("click",h);addEventListener("auxclick",h)})()`
}

export type PreviewLinkOpener = Readonly<{
  open: (href: unknown) => void
}>

/**
 * シェルの側で、受け取った URL を開く。
 * noopener 付きの window.open は、開けても仕様上 null を返すので、ブラウザに止められた
 * かどうかは見分けられない。そこで新しいタブで開いたときは、いつも onOpened で
 * 押せるリンク付きの知らせを出す（止められていても、利用者がそこから開ける）。
 */
export function createPreviewLinkOpener(
  options: Readonly<{
    window: Pick<Window, "open" | "location">
    onOpened: (url: string) => void
  }>
): PreviewLinkOpener {
  return {
    open(href) {
      const action = previewLinkAction(href, options.window.location.origin)
      if (action.kind === "ignore") return
      if (action.kind === "navigate") {
        options.window.location.assign(action.url)
        return
      }
      options.window.open(action.url, "_blank", "noopener,noreferrer")
      options.onOpened(action.url)
    },
  }
}

export type PreviewLinkShell = Readonly<{
  /** 対話の Preview（isolated execution）から届いた URL を開く */
  openLink: (href: unknown) => void
  /** 静的な HTML の Preview の文書を、リンクを受け取るスクリプト付きで作る */
  staticDocument: (html: string, networkAllowed: boolean) => string
  /** 静的な Preview を消したときに、受け口を外す */
  release: () => void
  dispose: () => void
}>

/**
 * シェル（Playground / Tour）の側の受け口。
 * - 静的な Preview には、描画のたびに新しい合言葉（nonce を兼ねる）を付けたスクリプトを入れ、
 *   その枠から同じ合言葉で届いたメッセージだけを受け取る。受け口（window の message）は、
 *   静的な Preview を出しているあいだだけ置く。
 * - 新しいタブで開いたときは、notice に押せるリンク付きの知らせを数秒出す。
 */
export function connectPreviewLinks(
  options: Readonly<{
    frame: HTMLIFrameElement
    notice: HTMLElement
    createDocument: (
      html: string,
      networkAllowed: boolean,
      linkToken: string
    ) => string
  }>
): PreviewLinkShell {
  const owner = options.frame.ownerDocument.defaultView!
  let token: string | undefined
  let hideTimer: ReturnType<typeof setTimeout> | undefined
  const showNotice = (url: string): void => {
    const parsed = new URL(url)
    const label = `${parsed.host}${parsed.pathname === "/" ? "" : parsed.pathname}`
    const link = options.notice.ownerDocument.createElement("a")
    link.href = url
    link.target = "_blank"
    link.rel = "noopener noreferrer"
    link.textContent = label.length > 60 ? `${label.slice(0, 59)}…` : label
    options.notice.replaceChildren("新しいタブで開く: ", link)
    options.notice.hidden = false
    if (hideTimer !== undefined) clearTimeout(hideTimer)
    hideTimer = setTimeout(() => {
      options.notice.hidden = true
    }, 8_000)
  }
  const opener = createPreviewLinkOpener({
    window: owner,
    onOpened: showNotice,
  })
  const onMessage = (event: MessageEvent): void => {
    if (event.source !== options.frame.contentWindow) return
    const data = event.data as {
      type?: unknown
      token?: unknown
      href?: unknown
    }
    if (data?.type !== "seseragi-preview-link") return
    if (token === undefined || data.token !== token) return
    opener.open(data.href)
  }
  let listening = false
  const release = (): void => {
    token = undefined
    if (!listening) return
    listening = false
    owner.removeEventListener("message", onMessage)
  }
  return {
    openLink: (href) => opener.open(href),
    staticDocument(html, networkAllowed) {
      token = crypto.randomUUID()
      if (!listening) {
        listening = true
        owner.addEventListener("message", onMessage)
      }
      return options.createDocument(html, networkAllowed, token)
    },
    release,
    dispose() {
      release()
      if (hideTimer !== undefined) clearTimeout(hideTimer)
    },
  }
}

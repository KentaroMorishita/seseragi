// 外部通信の許可を確かめるモーダル、止めたことの知らせ、許可の一覧（#1371）。
// ラベルは英語（Allow / Cancel / Remove）、説明の文は日本語（DESIGN.md「画面の言葉」）。
import type { NetworkPermissionStore } from "../runtime/network-permissions"
import { createIcon } from "./icons"

export type NetworkConsent = Readonly<{
  /** 許可を確かめる。Allow なら true。同時に二つは開かない */
  ask: (label: string) => Promise<boolean>
}>

export function createNetworkConsent(ownerDocument: Document): NetworkConsent {
  const dialog = ownerDocument.createElement("dialog")
  dialog.id = "network-consent-dialog"
  dialog.className = "pg-dialog network-consent"
  dialog.setAttribute("aria-labelledby", "network-consent-title")
  dialog.setAttribute("aria-describedby", "network-consent-lead")
  dialog.innerHTML = `
    <div class="network-consent-body">
      <h2 id="network-consent-title">外部と通信してよいですか</h2>
      <p id="network-consent-lead"></p>
      <ul>
        <li>入力した内容や、このブラウザに保存したデータが、外部へ送られることがあります。</li>
        <li>許可は、この例（または作業場所）についてだけ、このブラウザに覚えます。あとでコードを書き換えても、許可は残ります。</li>
        <li>Search の Settings にある Network access から、いつでも取り消せます。</li>
      </ul>
    </div>
    <form method="dialog" class="network-consent-actions">
      <button class="ui-button ui-button--ghost" value="cancel" type="submit">Cancel</button>
      <button class="ui-button ui-button--primary" value="allow" type="submit">Allow</button>
    </form>
  `
  ownerDocument.body.append(dialog)
  const lead = dialog.querySelector<HTMLElement>("#network-consent-lead")!
  let pending: Promise<boolean> | undefined
  return {
    ask(label) {
      if (pending !== undefined) return pending
      lead.textContent = `「${label}」は、外部のサイトから画像を読み込んだり、外部と通信したりします。`
      const opener = ownerDocument.activeElement as HTMLElement | null
      dialog.returnValue = "cancel"
      pending = new Promise<boolean>((resolve) => {
        dialog.addEventListener(
          "close",
          () => {
            pending = undefined
            opener?.focus?.({ preventScroll: true })
            resolve(dialog.returnValue === "allow")
          },
          { once: true }
        )
      })
      dialog.showModal()
      // 既定のフォーカスは、取り返しのつく Cancel に置く
      dialog.querySelector<HTMLButtonElement>('button[value="cancel"]')?.focus()
      return pending
    },
  }
}

export type NetworkNotice = Readonly<{
  show: () => void
  hide: () => void
}>

/** 止めたときに Preview の上へ一行で出す。Allow を押すと許可して実行し直す */
export function connectNetworkNotice(
  element: HTMLElement,
  onAllow: () => void
): NetworkNotice {
  const text = element.ownerDocument.createElement("span")
  text.textContent = "外部の画像と通信を止めています。"
  const allow = element.ownerDocument.createElement("button")
  allow.type = "button"
  allow.className = "ui-button ui-button--ghost ui-button--sm"
  allow.textContent = "Allow"
  allow.addEventListener("click", onAllow)
  element.replaceChildren(text, allow)
  element.hidden = true
  return {
    show: () => {
      element.hidden = false
    },
    hide: () => {
      element.hidden = true
    },
  }
}

/** Search の Settings から開く、許可の一覧。一つずつ取り消せる */
export function createNetworkSettings(
  ownerDocument: Document,
  store: NetworkPermissionStore
): Readonly<{ open: (opener?: HTMLElement) => void }> {
  const dialog = ownerDocument.createElement("dialog")
  dialog.id = "network-settings-dialog"
  dialog.className = "pg-dialog network-settings"
  dialog.setAttribute("aria-labelledby", "network-settings-title")
  const head = ownerDocument.createElement("header")
  head.className = "pg-dialog-head"
  const heading = ownerDocument.createElement("div")
  const title = ownerDocument.createElement("h2")
  title.id = "network-settings-title"
  title.textContent = "Network access"
  const lead = ownerDocument.createElement("p")
  lead.textContent =
    "外部の画像と通信について答えた例と作業場所です。取り消すと、次に要るときにもう一度聞きます。"
  heading.append(title, lead)
  const close = ownerDocument.createElement("button")
  close.type = "button"
  close.className = "pg-dialog-close"
  close.setAttribute("aria-label", "Close network access")
  close.append(createIcon(ownerDocument, "close"))
  close.addEventListener("click", () => dialog.close())
  head.append(heading, close)
  const list = ownerDocument.createElement("ul")
  list.className = "network-settings-list"
  dialog.append(head, list)
  ownerDocument.body.append(dialog)

  const render = (): void => {
    const permissions = store.list()
    if (permissions.length === 0) {
      const empty = ownerDocument.createElement("li")
      empty.className = "network-settings-empty"
      empty.textContent = "まだ、どの例にも答えていません。"
      list.replaceChildren(empty)
      return
    }
    list.replaceChildren(
      ...permissions.map((permission) => {
        const item = ownerDocument.createElement("li")
        item.dataset.scope = permission.scope
        const label = ownerDocument.createElement("span")
        label.className = "network-settings-label"
        label.textContent = permission.label
        const state = ownerDocument.createElement("span")
        state.className = `ui-tag${permission.decision === "allowed" ? " ui-tag--accent" : ""}`
        state.textContent =
          permission.decision === "allowed" ? "Allowed" : "Blocked"
        const remove = ownerDocument.createElement("button")
        remove.type = "button"
        remove.className = "ui-button ui-button--ghost ui-button--sm"
        remove.textContent = "Remove"
        remove.setAttribute("aria-label", `Remove ${permission.label}`)
        remove.addEventListener("click", () => store.remove(permission.scope))
        item.append(label, state, remove)
        return item
      })
    )
  }
  store.subscribe(() => {
    if (dialog.open) render()
  })
  let returnFocus: HTMLElement | undefined
  dialog.addEventListener("close", () => returnFocus?.focus())
  return {
    open(opener) {
      returnFocus = opener
      render()
      dialog.showModal()
      close.focus()
    },
  }
}

export type NetworkScope = Readonly<{ scope: string; label: string }>

export type NetworkGate = Readonly<{
  /** 今の例（作業場所、レッスン）で、外部通信を許可しているか */
  allowed: () => boolean
  /**
   * 実行の前に分かったとき: 外部通信が要り、まだ答えていなければ確かめる。
   * 戻り値は、この実行で外部通信を通すかどうか。止めたときは知らせを出す
   */
  resolve: (needsNetwork: boolean) => Promise<boolean>
  /** 実行してから分かったとき（Preview の中で CSP に止められた） */
  blocked: () => void
  /** Run を始めたときに、前の知らせを消す */
  reset: () => void
}>

export function createNetworkGate(
  options: Readonly<{
    store: NetworkPermissionStore
    consent: NetworkConsent
    notice: NetworkNotice
    current: () => NetworkScope
    /** 許可されたあとに、同じコードを実行し直す */
    rerun: () => void
  }>
): NetworkGate {
  const decision = () => options.store.get(options.current().scope)?.decision
  const ask = async (): Promise<boolean> => {
    const { scope, label } = options.current()
    const allowed = await options.consent.ask(label)
    // 答えている間に別の例へ移っていたら、その答えは使わない
    if (options.current().scope !== scope) return false
    options.store.set(scope, label, allowed ? "allowed" : "denied")
    return allowed
  }
  return {
    allowed: () => decision() === "allowed",
    async resolve(needsNetwork) {
      if (!needsNetwork) return decision() === "allowed"
      const current = decision()
      const allowed =
        current === undefined ? await ask() : current === "allowed"
      if (!allowed) options.notice.show()
      return allowed
    },
    blocked() {
      const current = decision()
      if (current === "allowed") return
      if (current === "denied") {
        options.notice.show()
        return
      }
      void ask().then((allowed) => {
        if (allowed) options.rerun()
        else options.notice.show()
      })
    },
    reset: () => options.notice.hide(),
  }
}

/** 知らせの Allow: 許可を覚えて実行し直す */
export function allowFromNotice(
  store: NetworkPermissionStore,
  current: () => NetworkScope,
  notice: NetworkNotice,
  rerun: () => void
): void {
  const { scope, label } = current()
  store.set(scope, label, "allowed")
  notice.hide()
  rerun()
}

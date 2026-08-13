import {
  type EditorPreferencesStore,
  isValidFixedFormatWidth,
  maximumFormatWidth,
  minimumFormatWidth,
} from "../preferences/editor-preferences"

type EditorSettingsOptions = Readonly<{
  button: HTMLButtonElement
  returnFocus?: HTMLButtonElement
  idPrefix: string
  store: EditorPreferencesStore
}>

export function connectEditorSettings({
  button,
  returnFocus = button,
  idPrefix,
  store,
}: EditorSettingsOptions): HTMLDialogElement {
  const ownerDocument = button.ownerDocument
  const dialog = ownerDocument.createElement("dialog")
  dialog.id = `${idPrefix}-settings-dialog`
  dialog.className = "editor-settings-dialog"
  dialog.setAttribute("aria-labelledby", `${idPrefix}-settings-title`)
  button.setAttribute("aria-controls", dialog.id)
  button.setAttribute("aria-expanded", "false")
  dialog.innerHTML = `
    <div class="editor-settings-heading">
      <div>
        <span>SHARED PREFERENCES</span>
        <h2 id="${idPrefix}-settings-title">Settings</h2>
      </div>
      <button class="editor-settings-close" type="button" aria-label="Settingsを閉じる">×</button>
    </div>
    <div class="editor-settings-content">
      <fieldset>
        <legend>Editor</legend>
        <label class="editor-setting-row">
          <span>
            <strong>Show indentation whitespace</strong>
            <small>Indent guidesとしてspaceとtabを表示します。</small>
          </span>
          <input class="editor-settings-whitespace" type="checkbox" />
        </label>
      </fieldset>
      <fieldset>
        <legend>Formatting</legend>
        <div class="editor-settings-width">
          <strong>Line width</strong>
          <label>
            <input class="editor-settings-auto" type="radio" name="${idPrefix}-format-width" value="auto" />
            <span><strong>Auto</strong><small>Format押下時のeditor幅に合わせます。</small></span>
          </label>
          <label>
            <input class="editor-settings-fixed" type="radio" name="${idPrefix}-format-width" value="fixed" />
            <span><strong>Fixed</strong><small>PlaygroundとTourで同じ幅を使用します。</small></span>
          </label>
          <label class="editor-settings-fixed-value">
            <span>Characters</span>
            <input
              class="editor-settings-width-input"
              type="number"
              inputmode="numeric"
              min="${minimumFormatWidth}"
              max="${maximumFormatWidth}"
              step="1"
              aria-describedby="${idPrefix}-settings-width-error"
            />
          </label>
          <p id="${idPrefix}-settings-width-error" class="editor-settings-error" role="alert" hidden></p>
        </div>
      </fieldset>
    </div>
  `
  ownerDocument.body.append(dialog)

  const closeButton = requiredDescendant(
    dialog,
    ".editor-settings-close",
    HTMLButtonElement
  )
  const whitespace = requiredDescendant(
    dialog,
    ".editor-settings-whitespace",
    HTMLInputElement
  )
  const auto = requiredDescendant(
    dialog,
    ".editor-settings-auto",
    HTMLInputElement
  )
  const fixed = requiredDescendant(
    dialog,
    ".editor-settings-fixed",
    HTMLInputElement
  )
  const fixedValue = requiredDescendant(
    dialog,
    ".editor-settings-width-input",
    HTMLInputElement
  )
  const error = requiredDescendant(
    dialog,
    ".editor-settings-error",
    HTMLElement
  )

  const validateFixedWidth = (): number | undefined => {
    const value = fixedValue.valueAsNumber
    const valid = isValidFixedFormatWidth(value)
    const message = valid
      ? ""
      : `${minimumFormatWidth}〜${maximumFormatWidth}の整数を入力してください。`
    fixedValue.setCustomValidity(message)
    error.textContent = message
    error.hidden = valid
    return valid ? value : undefined
  }

  const sync = (): void => {
    const preferences = store.get()
    whitespace.checked = preferences.showWhitespace
    auto.checked = preferences.formatWidth.mode === "auto"
    fixed.checked = preferences.formatWidth.mode === "fixed"
    fixedValue.value = String(preferences.formatWidth.fixed)
    fixedValue.disabled = !fixed.checked
    validateFixedWidth()
  }

  button.addEventListener("click", () => {
    sync()
    button.setAttribute("aria-expanded", "true")
    dialog.showModal()
    whitespace.focus()
  })
  closeButton.addEventListener("click", () => dialog.close())
  dialog.addEventListener("click", (event) => {
    if (event.target === dialog) dialog.close()
  })
  dialog.addEventListener("close", () => {
    button.setAttribute("aria-expanded", "false")
    returnFocus.focus()
  })
  whitespace.addEventListener("change", () => {
    store.update((preferences) => ({
      ...preferences,
      showWhitespace: whitespace.checked,
    }))
  })
  auto.addEventListener("change", () => {
    if (!auto.checked) return
    fixedValue.disabled = true
    store.update((preferences) => ({
      ...preferences,
      formatWidth: { ...preferences.formatWidth, mode: "auto" },
    }))
  })
  fixed.addEventListener("change", () => {
    if (!fixed.checked) return
    fixedValue.disabled = false
    const value = validateFixedWidth()
    if (value === undefined) {
      fixedValue.focus()
      return
    }
    store.update((preferences) => ({
      ...preferences,
      formatWidth: { mode: "fixed", fixed: value },
    }))
  })
  fixedValue.addEventListener("input", () => {
    const value = validateFixedWidth()
    if (value === undefined || !fixed.checked) return
    store.update((preferences) => ({
      ...preferences,
      formatWidth: { mode: "fixed", fixed: value },
    }))
  })
  store.subscribe(sync)

  return dialog
}

function requiredDescendant<T extends HTMLElement>(
  root: ParentNode,
  selector: string,
  elementType: { new (): T }
): T {
  const element = root.querySelector(selector)
  if (!(element instanceof elementType)) {
    throw new Error(`Missing required settings element: ${selector}`)
  }
  return element
}

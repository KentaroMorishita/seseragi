// 画面の小さなアイコン（16 × 16、線で描く）。文字で代用していた記号（＋F、×、› など）を置き換える。
// 中身は固定の文字列だけで、外から来る値は入れない。
const paths = {
  close: '<path d="m4.5 4.5 7 7M11.5 4.5l-7 7" />',
  chevron: '<path d="m6 4 4 4-4 4" />',
  file: '<path d="M4 2.5h5l3 3v8H4zM9 2.5v3h3" />',
  folder: '<path d="M2.5 4.5v8h11v-6.5H8L6.5 4.5z" />',
  newFile: '<path d="M4 2.5h5l3 3v8H4zM9 2.5v3h3M8 8v4M6 10h4" />',
  newFolder: '<path d="M2.5 4.5v8h11v-6.5H8L6.5 4.5zM8 7.5v4M6 9.5h4" />',
  collapse: '<path d="m5 6.5 3-3 3 3M5 12.5l3-3 3 3" />',
  entry:
    '<path d="M5.5 3.8v8.4a.5.5 0 0 0 .8.4l6-4.2a.5.5 0 0 0 0-.8l-6-4.2a.5.5 0 0 0-.8.4z" />',
  rename: '<path d="m10.5 3 2.5 2.5L6 12.5H3.5V10z" />',
  trash: '<path d="M3.5 4.5h9M6.5 4.5V3h3v1.5M5 4.5l.6 8.5h4.8l.6-8.5" />',
} as const

export type IconName = keyof typeof paths

export function iconMarkup(name: IconName): string {
  return `<svg class="ui-icon" viewBox="0 0 16 16" aria-hidden="true">${paths[name]}</svg>`
}

export function createIcon(ownerDocument: Document, name: IconName): Element {
  const template = ownerDocument.createElement("template")
  template.innerHTML = iconMarkup(name)
  return template.content.firstElementChild as Element
}

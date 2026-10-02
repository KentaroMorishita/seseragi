interface ReleaseCard {
  title: string
  detail: string
}

function escapeText(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
}

function card({ title, detail }: ReleaseCard): string {
  return `<section class="release-card"><h2>${escapeText(title)}</h2><p>${escapeText(detail)}</p></section>`
}

const cards: ReleaseCard[] = [
  { title: "Release <notes>", detail: "Search & navigation improvements" },
  { title: "Next release", detail: "Two smaller fixes" },
]

console.log(cards.map(card).join(""))
export {}

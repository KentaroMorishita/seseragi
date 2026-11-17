// 確認用: URL の # で表示状態を切り替える。カンマでつなげて組み合わせられる。
//   言語:   en（指定なしは日本語）
//   テーマ: dark（指定なしはライト）
//   案内:   quiet（初回の案内を畳んだ通常状態）
//   状態:   error（エラーがある） / complete（補完を開いている）
//   表示:   palette / output / map / code のどれか一つ
// 例: playground.html#en,dark,palette
// 本番では言語を URL のパス（/ と /en/）で、テーマを保存した設定で表します。# はモックの都合です。

const VIEWS = ["palette", "output", "map", "code"];
const TOGGLES = ["quiet", "error", "complete"];

const read = () => new Set(location.hash.slice(1).split(",").filter(Boolean));

const write = (parts) => {
  const order = ["en", "dark", ...TOGGLES, ...VIEWS];
  const hash = order.filter((part) => parts.has(part)).join(",");
  history.replaceState(null, "", hash ? `#${hash}` : location.pathname);
  apply();
};

const keep = (element, key, value) => {
  if (!(key in element.dataset)) element.dataset[key] = value ?? "";
};

function apply() {
  const parts = read();
  const en = parts.has("en");
  document.documentElement.lang = en ? "en" : "ja";
  document.body.dataset.state = [...parts].join(" ");

  for (const element of document.querySelectorAll("[data-en]")) {
    keep(element, "ja", element.textContent);
    element.textContent = en ? element.dataset.en : element.dataset.ja;
  }
  for (const element of document.querySelectorAll("[data-en-label]")) {
    keep(element, "jaLabel", element.getAttribute("aria-label"));
    element.setAttribute("aria-label", en ? element.dataset.enLabel : element.dataset.jaLabel);
  }
  for (const option of document.querySelectorAll("[data-lang]")) {
    option.setAttribute("aria-current", String((option.dataset.lang === "en") === en));
  }
  const view = VIEWS.find((name) => parts.has(name)) ?? "";
  for (const control of document.querySelectorAll("[data-current-view]")) {
    control.setAttribute("aria-current", String(control.dataset.currentView === view));
  }
}

document.addEventListener("click", (event) => {
  const target = event.target instanceof Element ? event.target : null;
  const lang = target?.closest("[data-lang]");
  const theme = target?.closest("[data-theme-toggle]");
  const view = target?.closest("[data-view]");
  const toggle = target?.closest("[data-toggle]");
  if (!lang && !theme && !view && !toggle) return;
  event.preventDefault();
  const parts = read();
  if (lang) {
    if (lang.dataset.lang === "en") parts.add("en");
    else parts.delete("en");
  }
  if (theme) {
    if (parts.has("dark")) parts.delete("dark");
    else parts.add("dark");
  }
  if (toggle) {
    const name = toggle.dataset.toggle;
    if (parts.has(name)) parts.delete(name);
    else parts.add(name);
  }
  if (view) {
    for (const name of VIEWS) parts.delete(name);
    if (view.dataset.view) parts.add(view.dataset.view);
  }
  write(parts);
});

addEventListener("hashchange", apply);
apply();

このsampleを選ぶ理由: Seseragiだけで、実際のproduct landing pageとして成立するWeb UIと、
見た目のsectionに対応するmulti-module sourceを一緒に読みたいときに選びます。

## この画面で試すこと

- HeroのPlayground / GitHub linkをkeyboardまたはpointerで開く。
- Readable / Composable / Aliveのtabを切り替える。
- chapterごとにcopy、accent、code presentationがSignal経由で差し替わることを確認する。
- desktop、tablet、mobileでhero、principles、chapter、image statement、CTAのflowを比べる。
- Explorerからvisual sectionとsource moduleの対応を追う。

## Module boundary

```text
main.ssrg                    entry / dom.run
app.ssrg                     URL準備とpage composition
url.ssrg                     String -> WebUrl
components/header.ssrg       brand navigation
components/hero.ssrg         first view
components/principles.ssrg   editorial principles
components/chapter/          state / Action / tabs / code panel
components/image-statement.ssrg
components/closing.ssrg
components/footer.ssrg
ui/styles.ssrg               shared named styles
```

`components/chapter/model.ssrg`はChapter state、Action、dispatchを内部に保持し、外には
`Signal<Html<Task<Unit>>>`だけを返します。`app.ssrg`はそのSignalへpage compositionをmapし、
`main.ssrg`はqueryと`dom.run`の実行境界へ責務を絞ります。

## Visual design

Heroはfixed HTTPS imageを構造の一部として使い、公式Seseragi logo、
`Code that keeps flowing.`、二つのprimary linkの優先順位を作ります。中盤はcard gridを
反復せず、sparseなprinciple list、情報量のあるinteractive code chapter、全幅river imageへ
密度を切り替え、closing CTAとfooterまで一つのpage rhythmを保ちます。

主要なfont sizeとsection spacingは`clamp`で変化し、gridは`auto-fit`と`minmax`でmobile固有の
一列flowへreflowします。page rootは横overflowを隠すだけに頼らず、各sectionがviewport内へ
収まるwidthを持ちます。code panelは短いsample lineと明示的indentでnarrow widthでも読めます。

## Source quality

見た目のために一つの巨大なsourceやtop-level `let`を増やさず、再利用するstyleとcomponentは
named `fn`にします。official logo、Unsplash image、Playground、repository URLは`url.ssrg`の
`String -> WebUrl`境界を通し、external anchorは`target: "_blank"`と
`rel: "noopener noreferrer"`を明示します。

## 前提と次のsample

`html-components`でHtml tree、`signal-run-route`で明示的な`dom.run`、
`feature-composition`でfeature-owned Signalを先に読むと追いやすくなります。form validationを
中心に見るなら`form-todo`、複数featureのapplication ownershipを見るなら`project-flow-app`へ進みます。

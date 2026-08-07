このsampleを選ぶ理由: Seseragiだけで、実際のプロダクトLPに近い完成度のWeb UIを組み立てたいときに選びます。
公式ロゴ、外部画像、レスポンシブなhero、複数section、typed Action、Signal、dom.run、コード表示までをsingle-fileでまとめています。

## この画面で試すこと

- HeroのPlayground / GitHubリンクを開く。
- Readable / Composable / Aliveのtabを切り替える。
- chapterごとにcopy、accent、code sampleがSignal経由で差し替わることを確認する。
- 狭い画面と広い画面で、hero、principle list、code panel、image statement、CTAの組み方がどう変わるかを見る。
- 画像、gradient、typography、code syntax colorをutility classへ逃がさずhtml.Styleで組み立てる例として読む。

## 構造

Chapterが表示中のsectionを表し、ActionはShowReadable / ShowComposable / ShowAliveだけを持ちます。
updateはActionを次のChapterへ畳み込むpure functionです。

runtime接続はdom.appではなく、signals.make、signals.map、dom.query、dom.runを明示しています。
MutableSignal<Chapter>を唯一のlocal stateとして持ち、viewからSignal<Html<Action>>を作ってdom.runへ渡します。
button clickはActionとしてdispatchされ、handleがsignals.updateへつなぎます。

## UIの読み方

Heroは固定HTTPSのUnsplash画像を全面背景に使い、その上へ公式Seseragi logo、copy、CTAを重ねています。
ロゴはassets/brand/source/seseragi-logo-dark.svgを使い、Playground / GitHubを含む外部linkはtarget: "_blank"とrel: "noopener noreferrer"を明示しています。

Why Seseragi feels differentはcard gridにせず、番号、罫線、見出し、本文だけのeditorial layoutにしています。
モバイルで見出しの語尾だけが次行へ落ちにくいよう、主要copyはclampで段階的に縮小します。

Three ways to see itではtabのselected stateに応じてcopy、accent、code sampleを切り替えます。
コード表示は単なるStringではなくtokenごとのspanでsyntax colorを持たせ、codeLineのindent引数で実際のSeseragi sourceに近い字下げを再現します。
横幅の狭い画面でも長い一行を無理にwrapさせないよう、showcase用のsample code自体を短く保っています。

後半は全幅のriver imageへcopyを重ね、最後のCTAとfooterへつなぎます。
LP全体を巨大な一つのviewへ直接書かず、hero、principles、chapterSection、imageStatement、closing、footerという画面上の意味単位でcomponent化しています。

## 前提と次のsample

html-componentsでHtml treeとcomponentの基本、signal-run-routeで明示的なdom.run接続を先に読むと追いやすくなります。
form eventやvalidationを含む完成UIへ進むならform-todo、複数moduleへ分割されたapplication構成を見るならproject-flow-appを選んでください。

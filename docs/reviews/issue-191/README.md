# Issue #191 Web UI visual regression review

`apps/playground/tests/fixtures/web-ui-regression.json`を、Web UI sample品質の
single review matrixとして使います。対象はcatalogの全`outputMode: html` sampleで、
各sampleに対して次を実ブラウザで確認します。

- desktop、568px landscape、iPhone 390px、Android 360px、minimum 320px
- Sample picker、mobile Code、Preview、multi-moduleのExplorer + tabs
- initial、route / feature / form / studio interaction、empty、disabled、image fallback
- pageとPreview iframeのhorizontal overflow、主要text/control contrast、image altと
  fallback layout

## Run locally

```sh
cd apps/playground
bun run test:visual:install
bun run test:visual
```

成功時にも`test-results/web-ui-review/`へsample / viewport / state別のPNGとHTML reportが
残ります。固定Unsplashの見た目そのものは#190のhuman review artifactで確認済みで、ここでは
networkに左右されないlocal SVGをrouteしてlayoutとstate差分を再現可能にします。failure route
では画像のnatural widthが0でもaltと2:1 layoutが残ることを確認します。

GitHub Actionsの[Web UI visual regression](../../../.github/workflows/web-ui-visual.yml)は
Chromium実行後に同directoryをartifactとしてuploadします。artifactを確認するときは、
initialだけでなくformのinvalid / valid / empty、projectのExplorer / day studio /
empty-disabled、comparison pairのriverside stateを並べて見ます。

## Reviewed baselines

代表stateはplatform別のcommitted screenshot baselineと自動比較します。localとCIの
verify commandはどちらも`bun run test:visual`です。意図した変更を反映するときだけ、
理由を明記して更新します。Linux版は同workflowの`update_reason`付きmanual dispatchで
生成し、Mac版と同じreview manifestへ記録します。

```sh
bun run test:visual:update -- "変更理由"
```

更新理由と各PNGのSHA-256は`e2e/visual-baselines.review.json`へ保存されます。差分失敗時の
expected / actual / diffは`test-results/web-ui-review/`から確認できます。

## Relationship to the existing evidence

#188と#189のartifactは各Showcaseのfeature動作を記録し、#190のartifactは全sampleのreal
Unsplash compositionを記録します。#191はそれらを置き換えず、viewport・state・surfaceを
machine-readableに揃えて、次の変更で同種の崩れをCI artifactから追えるようにします。

このsampleを選ぶ理由: Playgroundで試したsourceを、そのままlocal packageとして作成し、
VS Code、`seseragi dev`、production buildへ持ち出す最短経路を確認したいときに選びます。

## local projectを作る

install済みCLIから同じsourceを生成できます。scaffold用の別実装ではなく、このsampleの
`src/main.ssrg`と`src/app.ssrg`がCLIへ埋め込まれます。

```sh
seseragi new web hello-web
cd hello-web
seseragi dev --open
```

`app.ssrg`はstate、typed Action、pure `update`、Signal viewを所有します。`main.ssrg`は
`#app`を探し、完成したSignalをDOM runtimeへ渡すだけです。production outputは同じ
manifest targetから生成します。

```sh
seseragi build .
```

より短いsingle-file reducerは`interactive-app`、featureごとにstateを分ける構成は
`project-flow-app`を参照してください。

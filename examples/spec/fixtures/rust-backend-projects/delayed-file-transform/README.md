Clockで待機した後、input.txtを読み、processed付きの内容をoutput.txtへ書く。
待機中の取消では旧outputを保持し、scopeのsleepを使うfinalizerを待つ。

TSの通常process buildは従来のproviderを使う。nativeではClockの選択を明示する。

```sh
seseragi lock update .
seseragi experimental build-native . --entry main --out-dir native --provider std/clock::Clock=seseragi/runtime-native#clock-sleep
```

source-only準備中。WASM Clock row、Clock nowや一般provider ABIの対応を意味しない。

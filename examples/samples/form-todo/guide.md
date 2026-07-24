`onInput`と`onChange`はbrowserのEvent objectではなく、immutableな`InputEvent` / `ChangeEvent`を渡します。追加・削除・絞り込みはすべてpureな`update`で新しい`Model`を返します。

- text inputとtextareaは`event.value`をActionへ変換します。
- checkboxは`event.checked`を読み取ります。
- `onSubmit`を持つformはpage reloadを防いでからActionをdispatchします。
- 空のdraftはbuttonの`disabled`と`update`の両方で無視します。
- `Removed id`は`filter`で対象だけを除き、`FilterChanged`は元のitemsを失わず表示対象だけを切り替えます。

入力中も同じcontrolへfocusを戻すため、controlled inputとして続けて編集できます。日本語IMEの変換途中はrerenderを
保留し、確定した文字列だけを一度Actionへ変換します。Playgroundではtitleへ「日本語」、detailへ「カタカナ」を
変換入力し、候補選択・Backspace・caret移動のあとも文字列と選択位置が壊れないことを確認できます。

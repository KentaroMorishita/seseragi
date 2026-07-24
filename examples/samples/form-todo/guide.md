`onInput`と`onChange`はbrowserのEvent objectではなく、immutableな`InputEvent` / `ChangeEvent`を渡します。追加・削除・絞り込みはすべてpureな`update`で新しい`Model`を返します。

- text inputとtextareaは`event.value`をMsgへ変換します。
- checkboxは`event.checked`を読み取ります。
- `onSubmit`を持つformはpage reloadを防いでからMsgをdispatchします。
- 空のdraftはbuttonの`disabled`と`update`の両方で無視します。
- `Removed id`は`filter`で対象だけを除き、`FilterChanged`は元のitemsを失わず表示対象だけを切り替えます。

入力中も同じcontrolへfocusを戻すため、controlled inputとして続けて編集できます。

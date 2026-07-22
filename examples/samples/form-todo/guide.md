`onInput`と`onChange`はbrowserのEvent objectではなく、immutableな`InputEvent` / `ChangeEvent`を渡します。

- text inputとtextareaは`event.value`をMsgへ変換します。
- checkboxは`event.checked`を読み取ります。
- `onSubmit`を持つformはpage reloadを防いでからMsgをdispatchします。
- 空のdraftはbuttonの`disabled`と`update`の両方で無視します。

入力中も同じcontrolへfocusを戻すため、controlled inputとして続けて編集できます。

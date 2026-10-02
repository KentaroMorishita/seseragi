type Identity = {
  identity: string
  module: string
  namespace: string
  kind: string
}
type Reading = { en: string; ja: string }

// Correct only the two selected Process type explanations. Canonical signatures,
// descriptions, type classification and all unrelated fallback readings remain.
export function terminalReaderReading(
  item: Identity,
  fallback: Reading
): Reading {
  if (
    item.module !== "std/process" ||
    item.namespace !== "type" ||
    item.kind !== "opaque-type"
  )
    return fallback
  if (item.identity === "std/process::Process") {
    return {
      ja: "Processはprocess向けの実行環境が用意するサービスの型です。std/processをprocessという別名で読み込んだ場合、with process: process.Processで要求を宣言し、モジュールの読み取り操作を使います。Processを自分で作る公開コンストラクターはありません。",
      en: "Process is a service supplied by process-target execution. With std/process imported as process, declare with process: process.Process and use the module's read operations. There is no public Process constructor.",
    }
  }
  if (item.identity === "std/process::ProcessError") {
    return {
      ja: "ProcessErrorはプロセス情報の読み取りなどの失敗を表します。内部の表現は非公開ですが、InvalidEnvironmentNameなどの公開された形をmatchで区別できます。std/processをprocessという別名で読み込んだ場合、process.InvalidEnvironmentName nameで、拒否された設定名をnameとして受け取れます。",
      en: "ProcessError represents failures such as an unsuccessful process-information read. Its representation is private, but public alternatives such as InvalidEnvironmentName can be matched. With std/process imported as process, match process.InvalidEnvironmentName name to read the rejected setting name as name.",
    }
  }
  return fallback
}

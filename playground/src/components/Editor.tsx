import React, { useRef, useState } from "react"
import MonacoEditor, { Monaco } from "@monaco-editor/react"
import {
  initializeSeseragiLanguage,
  setSeseragiTheme,
} from "../lib/monaco-seseragi"

interface EditorProps {
  value: string
  onChange: (value: string) => void
}

const Editor: React.FC<EditorProps> = ({ value, onChange }) => {
  const monacoRef = useRef<Monaco | null>(null)
  const editorRef = useRef<any>(null)
  const [, setIsLanguageReady] = useState(false)

  const handleEditorDidMount = async (editor: any, monaco: Monaco) => {
    monacoRef.current = monaco
    editorRef.current = editor

    try {
      // Seseragi言語サポートを初期化（Monacoインスタンスを渡す）
      await initializeSeseragiLanguage(monaco)

      // Monaco言語リストを確認
      const languages = monaco.languages.getLanguages()
      console.log(
        "🔍 Is seseragi registered?",
        languages.some((l) => l.id === "seseragi")
      )

      // テーマを適用（Monacoインスタンスを渡す）
      setSeseragiTheme(monaco)

      // エディタの言語を強制設定
      const model = editor.getModel()
      if (model) {
        monaco.editor.setModelLanguage(model, "seseragi")
        console.log("🔍 Model language:", model.getLanguageId())
      }

      // 言語が準備完了
      setIsLanguageReady(true)

      console.log("🎨 Seseragi language and theme initialized")
    } catch (error) {
      console.error("❌ Failed to initialize Seseragi:", error)
    }
  }

  return (
    <MonacoEditor
      height="100%"
      language="seseragi"
      theme="seseragi-theme"
      value={value}
      onChange={(value) => onChange(value || "")}
      onMount={handleEditorDidMount}
      options={{
        minimap: { enabled: false },
        fontSize: 18,
        lineNumbers: "on",
        automaticLayout: true,
        scrollBeyondLastLine: false,
        wordWrap: "on",
        formatOnType: true,
        formatOnPaste: true,
        tabSize: 2,
        insertSpaces: true,
        detectIndentation: false,
      }}
    />
  )
}

export default Editor

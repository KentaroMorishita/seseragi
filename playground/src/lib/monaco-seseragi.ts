import { loadWASM } from "onigasm"
import { Registry } from "monaco-textmate"
import { wireTmGrammars } from "monaco-editor-textmate"

let isInitialized = false

export async function initializeSeseragiLanguage(
  monacoInstance?: any
): Promise<void> {
  if (isInitialized) return

  // Monacoインスタンスの取得
  const monaco = monacoInstance || (window as any).monaco
  if (!monaco) {
    throw new Error("Monaco Editor is not loaded")
  }

  try {
    // 1. OnigasmをWebAssemblyで初期化
    await loadWASM("/onigasm.wasm")

    // 2. JSONファイルをfetchで読み込み
    const [grammarResponse, configResponse] = await Promise.all([
      fetch("/seseragi.tmLanguage.json"),
      fetch("/language-configuration.json"),
    ])

    const [seseragiGrammar, languageConfig] = await Promise.all([
      grammarResponse.json(),
      configResponse.json(),
    ])

    // 3. Monaco言語登録
    monaco.languages.register({ id: "seseragi" })

    // 4. 言語設定（VS Code拡張と同じ）
    // wordPatternをRegExpオブジェクトに変換
    const config = {
      ...languageConfig,
      wordPattern: languageConfig.wordPattern
        ? new RegExp(languageConfig.wordPattern)
        : undefined,
    }
    monaco.languages.setLanguageConfiguration("seseragi", config)

    // 5. TextMateグラマーレジストリ作成
    const registry = new Registry({
      getGrammarDefinition: async (scopeName) => {
        if (scopeName === "source.seseragi") {
          return {
            format: "json",
            content: seseragiGrammar,
          }
        }
        throw new Error(`Unknown scope name: ${scopeName}`)
      },
    })

    // 6. グラマーをMonaco Editorに適用
    const grammars = new Map()
    grammars.set("seseragi", "source.seseragi")

    console.log("🔍 Grammar registry:", registry)
    console.log("🔍 Grammars map:", grammars)

    try {
      await wireTmGrammars(monaco, registry, grammars)
      console.log("🔍 TextMate grammars wired successfully")
    } catch (grammarError) {
      console.error("⚠️ Failed to wire TextMate grammars:", grammarError)
      // Fallback to basic syntax highlighting if TextMate fails
      console.log("⚠️ Falling back to basic syntax highlighting")
    }

    // 7. Sublime Material Dark テーマを適用
    monaco.editor.defineTheme("seseragi-theme", {
      base: "vs-dark",
      inherit: true,
      rules: [
        // Material Theme の正確な色（.tmTheme ファイルから）
        { token: "comment", foreground: "546E7A", fontStyle: "italic" },
        { token: "string", foreground: "C3E88D" },
        { token: "keyword", foreground: "C792EA" },
        { token: "number", foreground: "F78C6C" },
        { token: "operator", foreground: "89DDFF" },

        // Seseragi特有のトークン (Material Theme の正確な色)
        { token: "keyword.control.seseragi", foreground: "C792EA" },
        { token: "entity.name.type.primitive.seseragi", foreground: "FFCB6B" },
        {
          token: "entity.name.type.generic.seseragi",
          foreground: "FFCB6B",
          fontStyle: "italic",
        },
        { token: "keyword.operator.pipeline.seseragi", foreground: "89DDFF" },
        { token: "keyword.operator.bind.seseragi", foreground: "89DDFF" },
        { token: "entity.name.function.seseragi", foreground: "82AAFF" },
        { token: "support.function.builtin.seseragi", foreground: "82AAFF" },

        // 基本的なトークンタイプ (Material Theme の正確な色)
        {
          token: "comment.line.double-slash.seseragi",
          foreground: "546E7A",
          fontStyle: "italic",
        },
        { token: "string.quoted.double.seseragi", foreground: "C3E88D" },
        { token: "constant.numeric.seseragi", foreground: "F78C6C" },
        { token: "constant.language.boolean.seseragi", foreground: "F78C6C" },
        { token: "variable.other.seseragi", foreground: "EEFFFF" },
        { token: "storage.type.seseragi", foreground: "C792EA" },

        // 追加の Material Dark トークン
        { token: "entity.name.function", foreground: "82aaff" },
        { token: "entity.name.class", foreground: "f07178" },
        { token: "entity.name.type", foreground: "f07178" },
        { token: "variable.parameter", foreground: "f78c6c" },
        { token: "variable.language", foreground: "f07178" },
        { token: "support.constant", foreground: "89ddff" },
        { token: "support.function", foreground: "89ddff" },
        { token: "support.type", foreground: "89ddff" },
        { token: "support.class", foreground: "89ddff" },
        { token: "meta.tag", foreground: "f07178" },
        { token: "entity.name.tag", foreground: "f07178" },
        { token: "entity.other.attribute-name", foreground: "ffcb6b" },
        { token: "markup.underline", foreground: "80cbc4" },
        { token: "markup.italic", foreground: "c3e88d", fontStyle: "italic" },
        { token: "markup.bold", foreground: "c3e88d", fontStyle: "bold" },
        { token: "markup.heading", foreground: "c792ea", fontStyle: "bold" },
        { token: "markup.quote", foreground: "546e7a", fontStyle: "italic" },
        { token: "markup.list", foreground: "c3e88d" },
        { token: "markup.raw", foreground: "c3e88d" },
        { token: "markup.deleted", foreground: "ff5370" },
        { token: "markup.inserted", foreground: "c3e88d" },
        { token: "markup.changed", foreground: "ffcb6b" },
        { token: "constant.other.color", foreground: "89ddff" },
        { token: "constant.other.symbol", foreground: "89ddff" },
        { token: "constant.other.key", foreground: "89ddff" },
        { token: "keyword.other.unit", foreground: "f78c6c" },
        { token: "keyword.other.special-method", foreground: "82aaff" },
        { token: "keyword.other.new", foreground: "c792ea" },
        { token: "keyword.other.debugger", foreground: "ff5370" },
        { token: "keyword.other.important", foreground: "c792ea" },
        { token: "storage.modifier", foreground: "c792ea" },
        { token: "invalid", foreground: "ff5370" },
        { token: "invalid.deprecated", foreground: "ff5370" },
        { token: "invalid.illegal", foreground: "ff5370" },
      ],
      colors: {
        // Material Dark Theme の エディター色設定
        "editor.background": "#212121",
        "editor.foreground": "#eeffff",
        "editorLineNumber.foreground": "#424242",
        "editorLineNumber.activeForeground": "#848484",
        "editorCursor.foreground": "#ffcc00",
        "editor.selectionBackground": "#61616150",
        "editor.selectionHighlightBackground": "#ffcc0020",
        "editor.inactiveSelectionBackground": "#61616140",
        "editor.findMatchBackground": "#ffcc0020",
        "editor.findMatchHighlightBackground": "#ffcc0020",
        "editor.findRangeHighlightBackground": "#ffcc0020",
        "editor.hoverHighlightBackground": "#ffcc0020",
        "editor.lineHighlightBackground": "#00000030",
        "editor.rangeHighlightBackground": "#ffcc0020",
        "editorBracketMatch.background": "#ffcc0020",
        "editorBracketMatch.border": "#ffcc00",
        "editorCodeLens.foreground": "#84848470",
        "editorError.foreground": "#ff5370",
        "editorWarning.foreground": "#ffcb6b",
        "editorInfo.foreground": "#82aaff",
        "editorHint.foreground": "#89ddff",
        "editorGutter.background": "#212121",
        "editorGutter.modifiedBackground": "#82aaff",
        "editorGutter.addedBackground": "#c3e88d",
        "editorGutter.deletedBackground": "#ff5370",
        "editorGroup.border": "#2A2A2A",
        "editorGroupHeader.tabsBackground": "#212121",
        "editorGroupHeader.noTabsBackground": "#212121",
        "editorIndentGuide.background": "#42424260",
        "editorIndentGuide.activeBackground": "#424242",
        "editorRuler.foreground": "#42424260",
        "editorWhitespace.foreground": "#65737e",
        "editorWidget.background": "#1A1A1A",
        "editorWidget.foreground": "#eeffff",
        "editorWidget.border": "#2A2A2A",
        "editorHoverWidget.background": "#1A1A1A",
        "editorHoverWidget.border": "#2A2A2A",
        "editorSuggestWidget.background": "#1A1A1A",
        "editorSuggestWidget.border": "#2A2A2A",
        "editorSuggestWidget.foreground": "#eeffff",
        "editorSuggestWidget.highlightForeground": "#80cbc4",
        "editorSuggestWidget.selectedBackground": "#32424A",
        "peekView.background": "#1A1A1A",
        "peekView.border": "#2A2A2A",
        "peekViewEditor.background": "#212121",
        "peekViewEditor.matchHighlightBackground": "#ffcc0020",
        "peekViewResult.background": "#1A1A1A",
        "peekViewResult.fileForeground": "#eeffff",
        "peekViewResult.lineForeground": "#B0BEC5",
        "peekViewResult.matchHighlightBackground": "#ffcc0020",
        "peekViewResult.selectionBackground": "#32424A",
        "peekViewResult.selectionForeground": "#eeffff",
        "peekViewTitle.background": "#1A1A1A",
        "peekViewTitleDescription.foreground": "#B0BEC5",
        "peekViewTitleLabel.foreground": "#eeffff",
        "scrollbar.shadow": "#00000030",
        "scrollbarSlider.activeBackground": "#42424280",
        "scrollbarSlider.background": "#42424250",
        "scrollbarSlider.hoverBackground": "#42424270",
        "selection.background": "#80cbc4",
        "textBlockQuote.background": "#32424A",
        "textBlockQuote.border": "#2A2A2A",
        "textCodeBlock.background": "#1A1A1A",
        "textLink.activeForeground": "#89ddff",
        "textLink.foreground": "#80cbc4",
        "textPreformat.foreground": "#eeffff",
        "textSeparator.foreground": "#2A2A2A",
      },
    })

    isInitialized = true
    console.log("✅ Seseragi language support initialized successfully")
  } catch (error) {
    console.error("❌ Failed to initialize Seseragi language support:", error)
    throw error
  }
}

// 人気テーマのみを事前にインポート
const themes = {
  dracula: () => import("monaco-themes/themes/Dracula.json"),
  monokai: () => import("monaco-themes/themes/Monokai.json"),
  "github-dark": () => import("monaco-themes/themes/GitHub Dark.json"),
  "night-owl": () => import("monaco-themes/themes/Night Owl.json"),
  nord: () => import("monaco-themes/themes/Nord.json"),
  "solarized-dark": () => import("monaco-themes/themes/Solarized-dark.json"),
  "tomorrow-night": () => import("monaco-themes/themes/Tomorrow-Night.json"),
  cobalt: () => import("monaco-themes/themes/Cobalt.json"),
  "oceanic-next": () => import("monaco-themes/themes/Oceanic Next.json"),
}

// 利用可能なテーマ一覧を取得
export function getAvailableThemes() {
  return {
    "seseragi-theme": "Seseragi Material (Custom)",
    "vs-dark": "VS Code Dark",
    vs: "VS Code Light",
    dracula: "Dracula",
    monokai: "Monokai",
    "github-dark": "GitHub Dark",
    "night-owl": "Night Owl",
    nord: "Nord",
    "solarized-dark": "Solarized Dark",
    "tomorrow-night": "Tomorrow Night",
    cobalt: "Cobalt",
    "oceanic-next": "Oceanic Next",
  }
}

// テーマを動的に切り替え
export async function setTheme(
  themeName: string,
  monacoInstance?: any
): Promise<void> {
  const monaco = monacoInstance || (window as any).monaco
  if (!monaco) {
    throw new Error("Monaco Editor is not loaded")
  }

  // カスタムテーマまたは標準テーマの場合
  if (
    themeName === "seseragi-theme" ||
    themeName === "vs-dark" ||
    themeName === "vs"
  ) {
    monaco.editor.setTheme(themeName)
    return
  }

  // 事前定義されたテーマの場合
  if (themes[themeName as keyof typeof themes]) {
    try {
      const themeData = await themes[themeName as keyof typeof themes]()
      monaco.editor.defineTheme(themeName, themeData.default)
      monaco.editor.setTheme(themeName)
    } catch (error) {
      console.error(`Failed to load theme: ${themeName}`, error)
      monaco.editor.setTheme("vs-dark")
    }
  } else {
    console.warn(`Theme ${themeName} not available`)
    monaco.editor.setTheme("vs-dark")
  }
}

export function setSeseragiTheme(monacoInstance?: any): void {
  setTheme("seseragi-theme", monacoInstance)
}

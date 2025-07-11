import React, { useState, useCallback, useEffect } from "react"
import Editor from "./components/Editor"
import Output from "./components/Output"
import Toolbar from "./components/Toolbar"
import Resizer from "./components/Resizer"
import VerticalResizer from "./components/VerticalResizer"
import { compileAndRun } from "./lib/runner"
import { samples } from "./samples"

function App() {
  const [code, setCode] = useState(samples[0].code)
  const [output, setOutput] = useState<string>("")
  const [isRunning, setIsRunning] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [leftWidth, setLeftWidth] = useState(50)
  const [topHeight, setTopHeight] = useState(60)
  const [isMobile, setIsMobile] = useState(window.innerWidth <= 768)

  useEffect(() => {
    const handleResize = () => {
      setIsMobile(window.innerWidth <= 768)
    }
    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [])

  const handleRun = useCallback(async () => {
    setIsRunning(true)
    setError(null)
    setOutput("")

    try {
      const result = await compileAndRun(code)
      setOutput(result)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setIsRunning(false)
    }
  }, [code])

  const handleSampleChange = (sampleCode: string) => {
    setCode(sampleCode)
    setOutput("")
    setError(null)
  }

  return (
    <div className="app">
      <header className="header">
        <h1>Seseragi Playground</h1>
        <Toolbar
          onRun={handleRun}
          onSampleChange={handleSampleChange}
          isRunning={isRunning}
          samples={samples}
        />
      </header>
      <div className="main-content">
        {isMobile ? (
          // スマホサイズ: 縦並び
          <>
            <div className="editor-pane" style={{ height: `${topHeight}vh` }}>
              <Editor value={code} onChange={setCode} />
            </div>
            <VerticalResizer onResize={setTopHeight} />
            <div className="output-pane" style={{ height: `${100 - topHeight - 15}vh` }}>
              <div className="output-header">Output</div>
              <Output output={output} error={error} />
            </div>
          </>
        ) : (
          // デスクトップサイズ: 横並び
          <>
            <div className="editor-pane" style={{ width: `${leftWidth}%` }}>
              <Editor value={code} onChange={setCode} />
            </div>
            <Resizer onResize={setLeftWidth} />
            <div className="output-pane" style={{ width: `${100 - leftWidth}%` }}>
              <div className="output-header">Output</div>
              <Output output={output} error={error} />
            </div>
          </>
        )}
      </div>
    </div>
  )
}

export default App

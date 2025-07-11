import React, { useState, useCallback, useEffect } from "react"
import Editor from "./components/Editor"
import Output from "./components/Output"
import Toolbar from "./components/Toolbar"
import { compileAndRun } from "./lib/runner"
import { samples } from "./samples"

function App() {
  const [code, setCode] = useState(samples[0].code)
  const [output, setOutput] = useState<string>("")
  const [isRunning, setIsRunning] = useState(false)
  const [error, setError] = useState<string | null>(null)

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
        <div className="editor-pane">
          <Editor value={code} onChange={setCode} />
        </div>
        <div className="output-pane">
          <div className="output-header">Output</div>
          <Output output={output} error={error} />
        </div>
      </div>
    </div>
  )
}

export default App

import React, { useState } from "react"
import { getAvailableThemes, setTheme } from "../lib/monaco-seseragi"
import SampleModal from "./SampleModal"

interface Sample {
  name: string
  code: string
}

interface ToolbarProps {
  onRun: () => void
  onSampleChange: (code: string) => void
  isRunning: boolean
  samples: Sample[]
}

const Toolbar: React.FC<ToolbarProps> = ({
  onRun,
  onSampleChange,
  isRunning,
  samples,
}) => {
  const themes = getAvailableThemes()
  const [isModalOpen, setIsModalOpen] = useState(false)

  const handleThemeChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    try {
      await setTheme(e.target.value)
    } catch (error) {
      console.error("Failed to change theme:", error)
    }
  }

  return (
    <>
      <div className="toolbar">
        <button 
          className={`run-button ${isRunning ? 'running' : ''}`}
          onClick={onRun} 
          disabled={isRunning}
          title={isRunning ? "実行中..." : "コードを実行 (Ctrl+Enter)"}
        >
          {isRunning ? (
            <>
              <span className="spinner"></span>
              Running...
            </>
          ) : (
            "Run"
          )}
        </button>

        <button className="examples-button" onClick={() => setIsModalOpen(true)}>
          Examples
        </button>

        <select 
          className="theme-select"
          onChange={handleThemeChange} 
          defaultValue="seseragi-theme"
          title="テーマを選択"
        >
          {Object.entries(themes).map(([key, name]) => (
            <option key={key} value={key}>
              {name}
            </option>
          ))}
        </select>
      </div>

      <SampleModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onSelectSample={onSampleChange}
      />
    </>
  )
}

export default Toolbar

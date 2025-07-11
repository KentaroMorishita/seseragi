import React from "react"
import { getAvailableThemes, setTheme } from "../lib/monaco-seseragi"

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

  const handleThemeChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    try {
      await setTheme(e.target.value)
    } catch (error) {
      console.error("Failed to change theme:", error)
    }
  }

  return (
    <div className="toolbar">
      <select onChange={(e) => onSampleChange(e.target.value)}>
        {samples.map((sample, index) => (
          <option key={index} value={sample.code}>
            {sample.name}
          </option>
        ))}
      </select>

      <select onChange={handleThemeChange} defaultValue="seseragi-theme">
        {Object.entries(themes).map(([key, name]) => (
          <option key={key} value={key}>
            {name}
          </option>
        ))}
      </select>

      <button onClick={onRun} disabled={isRunning}>
        {isRunning ? "Running..." : "Run"}
      </button>
    </div>
  )
}

export default Toolbar
